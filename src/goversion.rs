use crate::consts::{DL_PAGE, DL_URL, FILE_EXT, GIT_VERSIONS, VERSION_LIST};
use crate::decompressor::ToDecompress;
use crate::error::Error;
// use crate::error::Result;
use crate::utils::get_local_version;
use anyhow::Context;

use anyhow::Result;
use git2::{Direction, Remote};

use itertools::Itertools;
use manic::Downloader;
use rayon::prelude::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use soup::prelude::*;
use soup::Soup;
use std::fmt;
use std::fmt::Formatter;
use std::io::{BufReader, Cursor, Write};
use std::path::{Path, PathBuf};
use tracing::instrument;
use tracing::{debug, error, warn};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoVersions {
    latest: GoVersion,
    pub versions: Vec<GoVersion>,
}

impl fmt::Display for GoVersions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Latest {}\nGot {} versions",
            self.latest,
            self.versions.len()
        )
    }
}

pub enum Downloaded {
    File { dir: PathBuf, vers: GoVersion },
    Mem { buf: Vec<u8>, vers: GoVersion },
}

impl Downloaded {
    #[instrument(err, ret, skip(self))]
    pub(crate) fn unpack(&self, path: &Path, rename: bool) -> Result<()> {
        let par = path.parent().ok_or(Error::PathBufErr)?;
        let vers = match self {
            Self::Mem { buf, vers } => {
                let mut r = ToDecompress::new(Cursor::new(buf))?;
                r.extract(path)?;
                vers
            }
            Self::File { dir, vers } => {
                let mut r = ToDecompress::new(BufReader::new(std::fs::File::open(dir)?))?;
                r.extract(path)?;
                vers
            }
        };
        if rename {
            std::fs::rename(par.join("go"), par.join(&format!("go{}", vers.version)))
                .map_err(Error::IOErr)
                .with_context(|| "Rename error".to_string())
        } else {
            Ok(())
        }
    }
}

impl GoVersions {
    #[instrument(err, ret(Display))]
    fn from_file(path: &Path) -> Result<Self> {
        let read = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&read)?)
    }
    #[instrument(err)]
    fn save(&self, list_path: Option<&Path>) -> Result<()> {
        let path = if let Some(p) = list_path {
            p
        } else {
            &VERSION_LIST
        };
        let mut file = std::fs::OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(path)?;
        let to_write = serde_json::to_string_pretty(&self)?;
        file.write_all(to_write.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }
    #[instrument(err, ret)]
    pub fn new(list_path: Option<&Path>) -> Result<Self> {
        // let client = Client::new();
        // let latest = Self::download_latest()?;
        // let path = if let Some(p) = list_path {
        //     p
        // } else {
        //     &VERSION_LIST
        // };
        // if path.exists() {
        //     let to_cmp = Self::from_file(path)?;
        //     if to_cmp.latest.version == latest.version {
        //         return Ok(to_cmp);
        //     }
        // }
        let mut vers = Self::download_versions()?;
        vers.par_sort_unstable_by(|a, b| b.version.cmp(&a.version));
        let latest = vers.first().cloned().ok_or(Error::NoVersion)?;
        let res = Self {
            versions: vers,
            latest,
        };
        res.save(list_path)?;
        Ok(res)
    }
    pub fn latest(&self) -> GoVersion {
        self.latest.clone()
    }

    #[instrument(ret)]
    /// Gets golang versions from git tags
    pub fn raw_git_versions() -> Result<Vec<String>> {
        let mut remote = Remote::create_detached("https://github.com/golang/go")?;
        let conn = remote.connect_auth(Direction::Fetch, None, None)?;
        let output = conn
            .list()?
            .iter()
            .map(|x| x.name().to_string())
            .filter(|x| x.starts_with("refs/tags/go"))
            .map(|x| x.replace("refs/tags/go", ""))
            .collect();
        Ok(output)
    }
    #[instrument(fields(before=%versions.join(",")))]
    pub fn parse_versions(versions: Vec<String>) -> Result<Vec<Version>> {
        enum IdealOrNot {
            Ideal(String),
            Not(Version),
        }
        let mut parsed: Vec<Version> = versions
            .par_iter()
            .filter_map(|x| {
                let split = x.split('.').collect_vec();
                if split.len() == 2 {
                    debug!("We got one without the one: {:#?}", split);
                    let major = split.get(0)?;
                    let minor = split.get(1)?;
                    debug!("Major: {}, minor: {}", major, minor);
                    Some(IdealOrNot::Not(semver::Version::new(
                        major.parse().ok()?,
                        minor.parse().ok()?,
                        0,
                    )))
                } else {
                    Some(IdealOrNot::Ideal(x.to_string()))
                }
            })
            .filter_map(|x| match x {
                IdealOrNot::Ideal(s) => Version::parse(&s).ok(),
                IdealOrNot::Not(v) => Some(v),
            })
            // .filter_map(|x| match x.pre.is_empty() {
            //     true => {
            //         if x.patch == 0 && x.minor == 18 {
            //             log!(Level::Info,  "Found one: {}", x);
            //         }
            //         Some(x)
            //     }
            //     false => None,
            // })
            .collect();
        parsed.par_sort_unstable_by(|a, b| b.cmp(a));
        Ok(parsed)
    }
    pub fn get_versions() -> Result<Vec<Version>> {
        let raw = GoVersions::raw_git_versions()?;
        GoVersions::parse_versions(raw)
    }
    pub fn download_latest() -> Result<GoVersion> {
        let latest = GIT_VERSIONS.first().ok_or(Error::NoVersion)?;
        let govers = Self::download_chosen(latest.clone())?;
        Ok(govers)
    }
    #[instrument(name = "parsing_versions", err(Display))]
    pub fn parsed_versions() -> Result<Vec<Version>> {
        let git = Self::raw_git_versions()?;
        let mut res = Vec::new();
        for i in git {
            let split = i.split('.').collect_vec();
            if split.len() < 3 {
                let minor = if let Some(s) = split.get(1) {
                    match s.parse::<u64>() {
                        Ok(d) => d,
                        Err(e) => {
                            error!("Failed to parse {} minor to u64 due to {}", s, e);
                            continue;
                        }
                    }
                } else {
                    0_u64
                };
                res.push(Version::new(split[0].parse()?, minor, 0));
            } else {
                match Version::parse(&i) {
                    Ok(s) => res.push(s),
                    Err(e) => {
                        error!("Failed to parse version {} due to {}", i, e);
                        paris::error!("Failed to parse version {} due to {}", i, e);
                        continue;
                    }
                }
            }
        }
        Ok(res)
    }
    /// Parses the versions into Versioning structs
    pub fn download_versions() -> Result<Vec<GoVersion>> {
        Ok(Self::raw_git_versions()?
            .par_iter()
            .filter_map(|x| {
                let sha = Self::sha(x, &DL_PAGE).ok();
                if let Some(s) = sha {
                    let url = Self::construct_url(x);
                    Some(GoVersion {
                        version: Version::parse(x).ok()?,
                        sha256: s,
                        dl_url: url,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<GoVersion>>())
    }
    /// Uses the soup library to extract the checksum from the golang download site
    fn sha(vers: impl std::fmt::Display, page: &str) -> Result<String> {
        let soup = Soup::new(page);
        let govers = format!("go{}", vers);
        let gofile = format!("{}.{}", govers, FILE_EXT.as_str());
        let latest = soup
            .tag("div")
            .attr("id", govers)
            .find()
            .ok_or(Error::NoSha)?;
        let mut children = latest.tag("tr").find_all().filter(|f| {
            let cls = f.get("class");
            if let Some(c) = cls {
                c != "first"
            } else {
                true
            }
        });
        let found = children
            .find(|child| {
                let res = child.class("filename").find();
                if let Some(s) = res {
                    s.text().contains(&gofile)
                } else {
                    false
                }
            })
            .ok_or(Error::NoSha)?;
        let sha = found.tag("tt").find().ok_or(Error::NoSha)?.text();
        Ok(sha)
    }
    pub fn download_chosen(vers: Version) -> Result<GoVersion> {
        let sha = Self::sha(&vers, &DL_PAGE)?;
        let url = Self::construct_url(&vers);
        Ok(GoVersion::new(vers, url, sha))
    }
    /// Constructs the url for the version
    fn construct_url(vers: impl std::fmt::Display) -> String {
        return format!("{}/go{}.{}", DL_URL, vers, FILE_EXT.as_str());
    }
    pub fn chosen_version(&self, vers: Version) -> Result<GoVersion> {
        let res = self
            .versions
            .par_iter()
            .find_any(|x| x.version == vers)
            .ok_or(Error::NoVersion)?;
        Ok(res.clone())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Golang version represented as a struct
pub struct GoVersion {
    /// Holds the golang version
    pub version: Version,
    /// Holds the download url for the version
    pub dl_url: String,
    /// Holds the sha256 checksum
    sha256: String,
}

impl fmt::Display for GoVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Version: {}\nDownload URL: {}\nSHA256: {}",
            self.version, self.dl_url, self.sha256
        )
    }
}

impl GoVersion {
    pub(crate) fn new(version: Version, dl_url: String, sha256: String) -> Self {
        Self {
            version,
            dl_url,
            sha256,
        }
    }
    /// Downloads the required version
    pub fn download(&self, output: Option<PathBuf>, workers: u8) -> Result<Downloaded> {
        let style = manic::ProgressStyle::default_bar()
            .template("{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-");
        let mut client = Downloader::new(&self.dl_url, workers)?;
        client.progress_bar();
        let hash = manic::Hash::new_sha256(self.sha256.to_string());
        client.verify(hash);
        client.bar_style(style);
        if let Some(path) = output {
            let path_str = path.to_str().ok_or(Error::PathBufErr)?;
            let filename = client.filename().to_string();
            client.download_and_save(path_str)?;
            Ok(Downloaded::File {
                dir: path.join(filename),
                vers: self.clone(),
            })
        } else {
            let res = client.download()?;
            Ok(Downloaded::Mem {
                buf: res.to_vec(),
                vers: self.clone(),
            })
        }
    }
    pub(crate) fn check_newer(&self, path: &Path) -> Result<bool> {
        if let Some(s) = get_local_version(path)? {
            if s < self.version {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(true)
        }
    }
}

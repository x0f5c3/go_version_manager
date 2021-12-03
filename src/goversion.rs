use crate::consts::{DEFAULT_INSTALL, DL_PAGE, DL_URL, FILE_EXT, GIT_VERSIONS, VERSION_LIST};
use crate::decompressor::ToDecompress;
use crate::error::Error;
use crate::error::Result;
use duct::cmd;
use git2::{Direction, Remote};
use manic::Downloader;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use soup::prelude::*;
use soup::Soup;
use std::io::{BufReader, Cursor, Write};
use std::path::{Path, PathBuf};
use versions::{SemVer, Versioning};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoVersions {
    latest: GoVersion,
    pub versions: Vec<GoVersion>,
}

pub enum Downloaded {
    File { dir: PathBuf, vers: GoVersion },
    Mem { buf: Vec<u8>, vers: GoVersion },
}

impl Downloaded {
    pub(crate) fn unpack(&self, path: &Path) -> Result<()> {
        match self {
            Self::Mem { buf, vers } => {
                let mut r = ToDecompress::new(Cursor::new(buf))?;
                r.extract(path)?;
                let par = path.parent().ok_or(Error::PathBufErr)?;
                std::fs::rename(par.join("go"), par.join(&format!("go{}", vers.version)))
                    .map_err(Error::IOError)
            }
            Self::File { dir, vers } => {
                let mut r = ToDecompress::new(BufReader::new(std::fs::File::open(dir)?))?;
                r.extract(path)?;
                let par = path.parent().ok_or(Error::PathBufErr)?;
                std::fs::rename(par.join("go"), par.join(&format!("go{}", vers.version)))
                    .map_err(Error::IOError)
            }
        }
    }
}

impl GoVersions {
    fn from_file(path: &Path) -> Result<Self> {
        let read = std::fs::read_to_string(path)?;
        serde_json::from_str(&read).map_err(Error::JSONErr)
    }
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
    pub fn new(list_path: Option<&Path>) -> Result<Self> {
        // let client = Client::new();
        let latest = Self::download_latest()?;
        let path = if let Some(p) = list_path {
            p
        } else {
            &VERSION_LIST
        };
        if path.exists() {
            let to_cmp = Self::from_file(path)?;
            if to_cmp.latest.version == latest.version {
                return Ok(to_cmp);
            }
        }
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
    pub fn check_local_latest(&self, path: Option<PathBuf>) -> Result<bool> {
        let local_vers = if let Some(p) = path {
            get_local_version(&p)?
        } else {
            get_local_version(&DEFAULT_INSTALL)?
        };
        if local_vers.is_none() {
            return Ok(false);
        }
        let somed = local_vers.unwrap();
        let latest = self.latest().version;
        if somed == latest {
            Ok(true)
        } else {
            Ok(false)
        }
    }
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
    pub fn parse_versions(versions: Vec<String>) -> Result<Vec<SemVer>> {
        let mut parsed: Vec<SemVer> = versions
            .iter()
            .filter_map(|x| Versioning::new(x.as_ref()))
            .filter_map(|x| match x {
                Versioning::Ideal(s) => Some(s),
                _ => None,
            })
            .collect();
        parsed.sort_unstable_by(|a, b| b.cmp(a));
        Ok(parsed)
    }
    pub fn download_latest() -> Result<GoVersion> {
        let latest = GIT_VERSIONS.first().ok_or(Error::NoVersion)?;
        let govers = Self::download_chosen(latest.clone())?;
        Ok(govers)
    }
    /// Parses the versions into Versioning structs
    pub fn download_versions() -> Result<Vec<GoVersion>> {
        Ok(GIT_VERSIONS
            .par_iter()
            .filter_map(|x| {
                let sha = Self::sha(x, &DL_PAGE).ok();
                if let Some(s) = sha {
                    let url = Self::construct_url(x);
                    Some(GoVersion {
                        version: x.clone(),
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
    pub fn download_chosen(vers: SemVer) -> Result<GoVersion> {
        let sha = Self::sha(&vers, &DL_PAGE)?;
        let url = Self::construct_url(&vers);
        Ok(GoVersion::new(vers, url, sha))
    }
    /// Constructs the url for the version
    fn construct_url(vers: impl std::fmt::Display) -> String {
        return format!("{}/go{}.{}", DL_URL, vers, FILE_EXT.as_str());
    }
    pub fn chosen_version(&self, vers: SemVer) -> Result<GoVersion> {
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
    pub version: SemVer,
    /// Holds the download url for the version
    pub dl_url: String,
    /// Holds the sha256 checksum
    sha256: String,
}

impl GoVersion {
    pub(crate) fn new(version: SemVer, dl_url: String, sha256: String) -> Self {
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
}

pub fn check_git() -> bool {
    match cmd!("git", "version").stdout_null().run() {
        Ok(_) => true,
        Err(e) => !matches!(e.kind(), std::io::ErrorKind::NotFound),
    }
}

pub fn get_local_version(path: &Path) -> Result<Option<SemVer>> {
    let output = cmd!(
        path.join("bin/go").to_str().ok_or(Error::PathBufErr)?,
        "version"
    )
    .read();
    if let Err(e) = output {
        return if e.kind() == std::io::ErrorKind::NotFound {
            Ok(None)
        } else {
            Err(e.into())
        };
    } else if let Ok(vers) = output {
        let version = vers.split(' ').nth(2);
        if version.is_none() {
            return Ok(None);
        }
        let somed = version.unwrap().replace("go", "");
        return Ok(SemVer::new(somed.as_ref()));
    }
    Ok(None)
}

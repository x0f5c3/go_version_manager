use crate::consts::{DEFAULT_INSTALL, DL_URL, FILE_EXT, GIT_VERSIONS, VERSION_LIST};
use crate::error::Error;
use crate::error::Result;
use crate::github::Tag;
use duct::cmd;
use git2::{Direction, Remote};
use indicatif::ParallelProgressIterator;
use manic::Client;
use manic::Downloader;
use rayon::prelude::*;
use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};
use soup::prelude::*;
use soup::Soup;
use std::path::{Path, PathBuf};
use versions::Versioning;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoVersions {
    pub versions: Vec<GoVersion>,
    latest: GoVersion,
}

pub enum Downloaded {
    File(PathBuf),
    Mem(Vec<u8>),
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
        serde_json::to_writer(&mut file, &self)?;
        file.sync_all()?;
        Ok(())
    }
    pub async fn new(git: bool, list_path: Option<&Path>) -> Result<Self> {
        let client = Client::new();
        let latest = Self::download_latest(git, &client).await?;
        let path = if let Some(p) = list_path {
            p
        } else {
            &VERSION_LIST
        };
        if path.exists() {
            let to_cmp = Self::from_file(path)?;
            if to_cmp.latest.version == latest {
                return Ok(to_cmp);
            }
        }
        let mut vers = Self::download_versions(git, &client).await?;
        vers.sort_unstable_by(|a, b| b.version.cmp(&a.version));
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
    pub async fn check_local_latest(&self, path: Option<PathBuf>) -> Result<bool> {
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
    pub async fn gh_versions(client: &Client) -> Result<Vec<Versioning>> {
        let resp: Vec<Tag> = client
            .get("https://api.github.com/repos/golang/go/tags?page=2&per_page=100")
            .header(USER_AGENT, "Get_Tag")
            .send()
            .await?
            .json()
            .await?;
        let mut filtered: Vec<Versioning> = resp
            .iter()
            .filter(|x| x.name.contains("go"))
            .map(|x| x.name.clone().replace("go", ""))
            .filter_map(|x| Versioning::new(x.as_ref()))
            .filter(|x| x.is_ideal())
            .collect::<Vec<_>>();
        filtered.par_sort_unstable_by(|a, b| b.cmp(a));
        Ok(filtered)
    }
    /// Gets golang versions from git tags
    fn raw_git_versions() -> Result<Vec<String>> {
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
    pub fn parse_versions(versions: Vec<String>) -> Result<Vec<Versioning>> {
        let mut parsed: Vec<Versioning> = versions
            .iter()
            .filter_map(|x| Versioning::new(x.as_ref()))
            .filter(|x| x.is_ideal())
            .collect();
        parsed.sort_unstable_by(|a, b| b.cmp(a));
        Ok(parsed)
    }
    pub async fn versions(git: bool, client: &Client) -> Result<Vec<Versioning>> {
        let res = if git {
            GIT_VERSIONS.clone()
        } else {
            Self::gh_versions(client).await?
        };
        Ok(res)
    }
    pub async fn download_latest(git: bool, client: &Client) -> Result<Versioning> {
        let vers = Self::versions(git, client).await?;
        let latest = vers.first().ok_or(Error::NoVersion)?;
        Ok(latest.clone())
    }
    /// Parses the versions into Versioning structs
    pub async fn download_versions(git: bool, client: &Client) -> Result<Vec<GoVersion>> {
        let parsed = Self::versions(git, client).await?;
        let page = client.get(DL_URL).send().await?.text().await?;
        Ok(parsed
            .par_iter()
            .progress_count(parsed.len() as u64)
            .filter_map(|x| {
                let sha = Self::sha(x, &page).ok();
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
        let gofile = format!("{}.{}", govers, FILE_EXT);
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
    /// Constructs the url for the version
    fn construct_url(vers: impl std::fmt::Display) -> String {
        return format!("{}/go{}.{}", DL_URL, vers, FILE_EXT);
    }
    pub fn chosen_version(&self, vers: Versioning) -> Result<GoVersion> {
        let res = self
            .versions
            .iter()
            .find(|x| x.version == vers)
            .ok_or(Error::NoVersion)?;
        Ok(res.clone())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Golang version represented as a struct
pub struct GoVersion {
    /// Holds the golang version
    pub version: Versioning,
    /// Holds the download url for the version
    pub dl_url: String,
    /// Holds the sha256 checksum
    sha256: String,
}

impl GoVersion {
    /// Downloads the required version async
    pub async fn download(&self, output: Option<PathBuf>, workers: u8) -> Result<Downloaded> {
        let style = manic::ProgressStyle::default_bar()
            .template("{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-");
        let mut client = Downloader::new(&self.dl_url, workers).await?;
        client.progress_bar();
        let hash = manic::Hash::new_sha256(self.sha256.to_string());
        client.verify(hash);
        client.bar_style(style);
        if let Some(path) = output {
            let path_str = path.to_str().ok_or(Error::PathBufErr)?;
            let filename = client.filename().to_string();
            client.download_and_save(path_str).await?;
            Ok(Downloaded::File(path.join(filename)))
        } else {
            let res = client.download().await?;
            Ok(Downloaded::Mem(res.to_vec().await))
        }
    }
}

pub fn check_git() -> bool {
    match cmd!("git", "version").stdout_null().run() {
        Ok(_) => true,
        Err(e) => !matches!(e.kind(), std::io::ErrorKind::NotFound),
    }
}

pub fn get_local_version(path: &Path) -> Result<Option<Versioning>> {
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
        return Ok(Versioning::new(somed.as_ref()));
    }
    Ok(None)
}

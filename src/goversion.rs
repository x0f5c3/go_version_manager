use crate::consts::{DL_URL, FILE_EXT};
use crate::error::Error;
use crate::github::Tag;
use duct::cmd;
use manic::Client;
use manic::Downloader;
use reqwest::header::USER_AGENT;
use soup::prelude::*;
use soup::Soup;
use std::path::PathBuf;
use versions::Versioning;
use rayon::prelude::*;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use indicatif::ParallelProgressIterator;


pub struct GoVersions {
    pub versions: Vec<GoVersion>,
    latest: GoVersion,
}


pub enum Downloaded {
    File(PathBuf),
    Mem(Vec<u8>),
}


impl GoVersions {
    pub async fn new(git: bool) -> Result<Self> {
        let client = Client::new();
        let mut vers = Self::download_versions(git, &client).await?;
        vers.sort_by(|a, b| b.version.cmp(&a.version));
        let latest = vers.first().cloned().ok_or(Error::NoVersion)?;
        Ok(Self {
            versions: vers,
            latest,
        })
    }
    pub fn latest(&self) -> GoVersion {
        self.latest.clone()
    }
    pub async fn check_local_latest(&self) -> Result<bool> {
        let local_vers = get_local_version()?;
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
        filtered.sort_unstable();
        filtered.reverse();
        Ok(filtered)
    }
    /// Gets golang versions from git tags
    fn raw_git_versions() -> Result<Vec<String>> {
        let output: Vec<String> =
            cmd!("git", "ls-remote", "--tags", "https://github.com/golang/go")
                .read()?
                .trim()
                .lines()
                .filter(|x| x.contains("go"))
                .filter_map(|x| x.split('\t').nth(1))
                .filter_map(|x| x.split('/').nth(2))
                .map(|x| x.replace("go", ""))
                .collect();
        Ok(output)
    }
    pub fn parse_versions(versions: Vec<String>) -> Result<Vec<Versioning>> {
        let mut parsed: Vec<Versioning> = versions
            .iter()
            .filter_map(|x| Versioning::new(x.as_ref()))
            .filter(|x| x.is_ideal())
            .collect();
        parsed.sort_unstable();
        parsed.reverse();
        Ok(parsed)
    }
    /// Parses the versions into Versioning structs
    pub async fn download_versions(git: bool, client: &Client) -> Result<Vec<GoVersion>> {
        let mut parsed: Vec<Versioning> = if git {
            let unparsed = Self::raw_git_versions()?;
            Self::parse_versions(unparsed)?
        } else {
            Self::gh_versions(client).await?
        };
        parsed.sort_unstable();
        parsed.reverse();
        let page = client.get(DL_URL).send().await?.text().await?;
        Ok(parsed.par_iter().progress_count(parsed.len() as u64).filter_map(|x| {
            let sha = Self::sha(x, &page).ok();
            if let Some(s) = sha {
                let url = Self::construct_url(x);
                Some(GoVersion{
                    version: x.clone(),
                    sha256: s,
                    dl_url: url,
                })
            } else {
                None
            }
        }).collect::<Vec<GoVersion>>())
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
        let res = self.versions.iter().find(|x| x.version == vers).ok_or(Error::NoVersion)?;
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
        let style = indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-");
        let mut client = Downloader::new(&self.dl_url, workers).await?;
        client.progress_bar();
        let hash = manic::Hash::SHA256(self.sha256.to_string());
        client.verify(hash);
        client.bar_style(style);
        if let Some(path) = output {
            let path_str = path.to_str().ok_or(Error::PathBufErr)?;
            let filename = client.filename().to_string();
            client.download_and_save(path_str, true).await?;
            Ok(Downloaded::File(path.join(filename)))
        } else {
            let res = client.download_and_verify().await?;
            Ok(Downloaded::Mem(res))
        }
    }
}

pub fn check_git() -> bool {
    match cmd!("git", "version").stdout_null().run() {
        Ok(_) => true,
        Err(e) => !matches!(e.kind(), std::io::ErrorKind::NotFound),
    }
}

pub fn get_local_version() -> Result<Option<Versioning>> {
    let output = cmd!("go", "version").read();
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


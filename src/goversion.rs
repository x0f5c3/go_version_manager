use crate::consts::{DL_URL, FILE_EXT};
use crate::error::Error;
use crate::github::Tag;
use duct::cmd;
use indicatif::ProgressBar;
use manic::progress::downloader;
use reqwest::header::USER_AGENT;
use soup::prelude::*;
use soup::Soup;
use std::path::PathBuf;
use versions::Versioning;

/// Golang version represented as a struct
pub struct GoVersion {
    /// Holds the golang version
    pub version: Versioning,
    /// Holds the download url for the version
    dl_url: String,
    /// Holds the sha256 checksum
    sha256: String,
}

impl GoVersion {
    /// Gets golang versions from git tags
    fn get_git_versions() -> Result<Vec<String>, Error> {
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
    pub async fn get_gh_version() -> Result<Vec<Versioning>, Error> {
        let client = reqwest::Client::new();
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
    /// Parses the versions into Versioning structs
    pub fn get_versions() -> Result<Vec<Versioning>, Error> {
        let unparsed = Self::get_git_versions()?;
        let mut parsed: Vec<Versioning> = unparsed
            .iter()
            .filter_map(|x| Versioning::new(x.as_ref()))
            .filter(|x| x.is_ideal())
            .collect();
        parsed.sort_unstable();
        parsed.reverse();
        Ok(parsed)
    }
    /// Gets the latest versions by sorting the parsed versions
    async fn get_latest(git: bool) -> Result<Versioning, Error> {
        let mut versions = if git {
            Self::get_versions()?
        } else {
            Self::get_gh_version().await?
        };
        versions.sort_by(|a, b| b.cmp(&a));
        let latest = versions.first().ok_or(Error::NoVersion)?.to_owned();
        Ok(latest)
    }
    /// Uses the soup library to extract the checksum from the golang download site
    async fn get_sha(vers: impl std::fmt::Display) -> Result<String, Error> {
        let resp = reqwest::get(DL_URL).await?;
        let soup = Soup::new(&resp.text().await?);
        let govers = format!("go{}", vers);
        let gofile = format!("{}.{}", govers, FILE_EXT);
        println!("{}", gofile);
        let latest = soup
            .tag("div")
            .attr("id", govers)
            .find()
            .ok_or(Error::NoSha)?;
        println!("Found latest");
        let mut children = latest.tag("tr").find_all().filter(|f| {
            let cls = f.get("class");
            if cls.is_some() {
                if cls.unwrap() == "first" {
                    false
                } else {
                    true
                }
            } else {
                true
            }
        });
        let found = children
            .find(|child| {
                child
                    .class("filename")
                    .find()
                    .unwrap()
                    .text()
                    .contains(&gofile)
            })
            .ok_or(Error::NoSha)?;
        println!("Found filename");
        let sha = found.tag("tt").find().ok_or(Error::NoSha)?.text();
        Ok(sha)
    }
    /// Constructs the url for the version
    fn construct_url(vers: impl std::fmt::Display) -> String {
        return format!("{}/go{}.{}", DL_URL, vers, FILE_EXT);
    }
    /// Downloads the required version async
    pub async fn download(&self, output: PathBuf, workers: u8) -> Result<PathBuf, Error> {
        let style = indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-");
        let path_str = output.to_str().ok_or(Error::PathBufErr)?;
        let pb = ProgressBar::new(100);
        pb.set_style(style);
        let client = reqwest::Client::new();
        let filename = manic::downloader::get_filename(&self.dl_url)?;
        downloader::download_verify_and_save(
            &client,
            &self.dl_url,
            workers,
            &self.sha256,
            path_str,
            pb,
        )
        .await?;
        Ok(output.join(filename))
    }
    /// Constructs the latest GoVersion
    pub async fn latest(git: bool) -> Result<Self, Error> {
        let vers = GoVersion::get_latest(git).await?;
        let url = GoVersion::construct_url(&vers);
        let sha = GoVersion::get_sha(&vers).await?;
        Ok(GoVersion {
            version: vers,
            dl_url: url,
            sha256: sha,
        })
    }
    pub async fn version(vers: Versioning) -> Result<Self, Error> {
        let url = GoVersion::construct_url(&vers);
        let sha = GoVersion::get_sha(&vers).await?;
        Ok(GoVersion {
            version: vers,
            dl_url: url,
            sha256: sha,
        })
    }
}

pub fn check_git() -> bool {
    match cmd!("git", "version").stdout_null().run() {
        Ok(_) => true,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => false,
            _ => true,
        },
    }
}

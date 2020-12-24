
use indicatif::ProgressBar;
use soup::prelude::*;
use soup::Soup;
use std::path::PathBuf;
use versions::Versioning;
use crate::error::Error;
use manic::progress::downloader;
use duct::cmd;
#[cfg(target_os = "linux")]
static FILE_EXT: &str = "linux-amd64.tar.gz";
#[cfg(target_os = "windows")]
static FILE_EXT: &str = "windows-amd64.msi";
#[cfg(target_os = "macos")]
static FILE_EXT: &str = "darwin-amd64.pkg";

static DL_URL: &str = "https://golang.org/dl";

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
        let output: Vec<String> = cmd!("git", "ls-remote", "--tags", "https://github.com/golang/go").read()?
        .trim()
        .lines()
        .filter(|x| x.contains("go"))
        .filter_map(|x| x.split('\t').nth(1))
        .filter_map(|x| x.split('/').nth(2))
        .map(|x| x.replace("go", ""))
        .collect();
        Ok(output)
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
    fn get_latest() -> Result<Versioning, Error> {
        let mut versions = GoVersion::get_versions()?;
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
        let latest = soup.tag("div").attr("id", govers).find().ok_or(Error::NoSha)?;
        let mut children = latest.tag("tr").class("highlight").find_all();
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
        let sha = found.tag("tt").find().ok_or(Error::NoSha)?.text();
        Ok(sha)
    }
    /// Constructs the url for the version
    fn construct_url(vers: impl std::fmt::Display) -> String {
        format!("{}/go{}.{}", DL_URL, vers, FILE_EXT)
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
        downloader::download_verify_and_save(&client, &self.dl_url, workers, &self.sha256, path_str, pb).await?;
        Ok(output.join(filename))
    }
    /// Constructs the latest GoVersion
    pub async fn latest() -> Result<Self, Error> {
        let vers = GoVersion::get_latest()?;
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
        Ok(_) => return true,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => return false,
            _ => return true,
        }
    }
}

use duct::cmd;
use versions::Versioning;
use sha2::Sha256;
use indicatif::ProgressBar;
use std::path::PathBuf;
use soup::Soup;
use reqwest::Url;
use std::fs::File;
use std::{fmt,error};
use soup::{NodeExt, QueryBuilderExt};
use std::io::ErrorKind;



static DL_URL: &str = "https://golang.org/dl";

#[cfg(target_os = "linux")]
static FILE_EXT: &str = "linux-amd64.tar.gz";
#[cfg(target_os = "windows")]
static FILE_EXT: &str = "windows-amd64.msi";
#[cfg(target_os = "macos")]
static FILE_EXT: &str = "darwin-amd64.pkg";


#[derive(Debug, Clone)]
struct WrongSha;

impl fmt::Display for WrongSha {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Checksum doesn't match")
    }
}

impl error::Error for WrongSha {}

/// Golang version represented as a struct
struct GoVersion {
    /// Holds the golang version
    pub version: Versioning,
    /// Holds the download url for the version
    dl_url: Url,
    /// Holds the sha256 checksum
    sha256: String,
}

impl GoVersion {
    /// Gets golang versions from git tags
    fn get_git_versions() -> Vec<String> {
        let output = cmd!("git", "ls-remote", "--tags", "https://github.com/golang/go")
            .read()
            .unwrap();
        let output = output.trim();
        let tags: Vec<String> = output
            .lines()
            .filter(|x| x.contains("go"))
            .map(|x| x.split('\t').nth(1).unwrap())
            .map(|x| x.split('/').nth(2).unwrap())
            .map(|x| x.replace("go", ""))
            .collect();
        tags
    }
    /// Parses the versions into Versioning structs
    fn get_versions() -> Vec<Versioning> {
        let unparsed = Self::get_git_versions();
        let parsed: Vec<Versioning> = unparsed
            .iter()
            .map(|x| Versioning::new(x.as_ref()).unwrap())
            .filter(|x| x.is_ideal())
            .collect();
        parsed
    }
    /// Gets the latest versions by sorting the parsed versions
    fn get_latest() -> Option<Versioning> {
        let mut versions = GoVersion::get_versions();
        versions.sort_by(|a, b| b.cmp(&a));
        let latest = versions.first()?.to_owned();
        Some(latest)
    }
    /// Uses the soup library to extract the checksum from the golang download site
    async fn get_sha(vers: impl std::fmt::Display) -> Result<String> {
        let resp = reqwest::get(DL_URL).await?;
        let soup = Soup::new(&resp.text().await?);
        let govers = format!("go{}", vers);
        let gofile = format!("{}.{}", govers, FILE_EXT);
        let latest = soup.tag("div").attr("id", govers.clone()).find().unwrap();
        let children = latest.tag("tr").class("highlight").find_all();
        let found = children
            .filter(|child| {
                child
                    .class("filename")
                    .find()
                    .unwrap()
                    .text()
                    .contains(&gofile)
            })
            .next()
            .unwrap();
        let sha = found.tag("tt").find().unwrap().text();
        Ok(sha)
    }
    /// Constructs the url for the version
    fn construct_url(vers: impl std::fmt::Display) -> Url {
        let ret = Url::parse(&format!("{}/go{}.{}", DL_URL, vers, FILE_EXT)).unwrap();
        ret
    }
    /// Downloads the required version async
    pub async fn download(&self, output: PathBuf) -> Result<PathBuf> {
        let style = indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-");
        let mut resp = reqwest::get(self.dl_url.clone()).await?;
        let total = resp.content_length().unwrap();
        let pb = ProgressBar::new(total);
        pb.set_style(style);
        let mut hash = Sha256::new();
        let mut f = File::create(output.clone())?;
        while let Some(chunk) = resp.chunk().await? {
            let len = chunk.len();
            f.write(&chunk)?;
            pb.inc(len as u64);
            hash.update(&chunk);
        }
        f.flush()?;
        f.sync_all()?;
        pb.finish();
        let finally = hash.finalize();
        let hexed = format!("{:x}", finally);
        if self.sha256 != hexed {
            return Err(WrongSha.into());
        }
        Ok(output)
    }
    /// Constructs the latest GoVersion
    pub async fn latest() -> Option<Self> {
        let vers = GoVersion::get_latest()?;
        let url = GoVersion::construct_url(&vers);
        let sha = GoVersion::get_sha(&vers).await.unwrap();
        Some(GoVersion {
            version: vers,
            dl_url: url,
            sha256: sha,
        })
    }
}

fn check_git() -> bool {
    match cmd!("git", "version").run() {
        Ok(_) => return true,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => return false,
            _ => return true,
        },
    }
}

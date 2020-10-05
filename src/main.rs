use duct::cmd;
use human_panic::setup_panic;
use indicatif::ProgressBar;
use sha2::{Digest, Sha256};
use reqwest::Url;
use soup::prelude::*;
use soup::Soup;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::io::prelude::*;
use versions::Versioning;
use directories::UserDirs;
#[cfg(target_os = "linux")]
static FILE_EXT: &str = "linux-amd64.tar.gz";
#[cfg(target_os = "windows")]
static FILE_EXT: &str = "windows-amd64.msi";
#[cfg(target_os = "macos")]
static FILE_EXT: &str = "darwin-amd64.pkg";

static DL_URL: &str = "https://golang.org/dl";
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_panic!();
    let golang = GoVersion::latest().await;
    let file_path = golang.download().await?;
    let mut f = archiver_rs::open(file_path.as_path())?;
    let dst = std::path::Path::new("/home/xc5/Downloads/go_test");
    f.extract(&dst)?;
    
    
    Ok(())
}

struct GoVersion {
    version: Versioning,
    dl_url: Url,
    sha256: String,
}

impl GoVersion {
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
    fn get_versions() -> Vec<Versioning> {
        let unparsed = Self::get_git_versions();
        let mut parsed: Vec<Versioning> = unparsed
            .iter()
            .map(|x| Versioning::new(x.as_ref()).unwrap())
            .filter(|x| x.is_ideal())
            .collect();
        parsed
    }
    fn get_latest() -> Versioning {
        let mut versions = GoVersion::get_versions();
        versions.sort_by(|a, b| b.cmp(&a));
        versions.first().unwrap().to_owned()
    }
    async fn get_sha(vers: impl std::fmt::Display) -> Result<String, Box<dyn Error>> {
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
    fn construct_url(vers: impl std::fmt::Display) -> Url {
        let ret = Url::parse(&format!("{}/go{}.{}", DL_URL, vers, FILE_EXT)).unwrap();
        ret
    }
    pub async fn download(&self) -> Result<PathBuf, Box<dyn Error>> {
        let style = indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-");
        let mut resp = reqwest::get(self.dl_url.clone()).await?;
        let total = resp.content_length().unwrap();
        let user_dirs = UserDirs::new().unwrap();
        let download_dir = user_dirs.download_dir().unwrap();
        let filename = resp.url().path_segments().unwrap().last().unwrap();
        let path = download_dir.join(filename);
        let pb = ProgressBar::new(total);
        pb.set_style(style);
        let mut hash = Sha256::new();
        let mut f = File::create(path.clone())?;
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
        println!("{}", self.sha256);
        let hexed = format!("{:x}", finally);
        if self.sha256 == hexed {
            println!("Nice");
        }
        Ok(path)
    }
    pub async fn latest() -> Self {
        let vers = GoVersion::get_latest();
        let url = GoVersion::construct_url(&vers);
        let sha = GoVersion::get_sha(&vers).await.unwrap();
        GoVersion {
            version: vers,
            dl_url: url,
            sha256: sha,
        }
    }
}

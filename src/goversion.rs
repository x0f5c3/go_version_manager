use crate::consts::{ARCH, CLIENT, DOWNLOAD_URL, VERSION_LIST};
use crate::decompressor::ToDecompress;
use crate::utils::get_local_version;
use anyhow::Context;

use anyhow::Result;

use manic::Downloader;
use rayon::prelude::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;
use std::io::{BufReader, Cursor, Write};
use std::path::{Path, PathBuf};
use tracing::instrument;

pub const DLURL: &str = "https://go.dev/dl/?mode=json&include=all";

const KIND: &str = "archive";

#[cfg(target_os = "windows")]
const OS: &str = "windows";

#[cfg(target_os = "macos")]
const OS: &str = "darwin";

#[cfg(target_os = "linux")]
const OS: &str = "linux";

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct File {
    filename: String,
    os: String,
    arch: String,
    sha256: String,
    size: String,
    kind: String,
}

impl File {
    pub fn get_url(&self) -> String {
        format!("{}/{}", DOWNLOAD_URL, self.filename)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GoVersion {
    pub version: String,
    #[serde(skip, default = "default_version")]
    pub parsed: Version,
    stable: bool,
    #[serde(skip)]
    is_parsed: bool,
    files: Vec<File>,
}

impl fmt::Display for GoVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version)
    }
}

impl Default for GoVersion {
    fn default() -> Self {
        Self {
            version: String::new(),
            parsed: default_version(),
            stable: false,
            is_parsed: false,
            files: Vec::new(),
        }
    }
}

fn default_version() -> Version {
    Version::new(0, 0, 0)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoVersions {
    latest: GoVersion,
    path: PathBuf,
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

impl Default for GoVersions {
    fn default() -> Self {
        GoVersions::new(VERSION_LIST.clone()).unwrap()
    }
}

pub enum Downloaded {
    File { dir: PathBuf, vers: GoVersion },
    Mem { buf: Vec<u8>, vers: GoVersion },
}

impl Downloaded {
    #[instrument(err, ret, skip(self))]
    pub(crate) fn unpack(&self, path: &Path, rename: bool) -> Result<()> {
        let par = path.parent().context("No parent")?;
        let vers = match self {
            Self::Mem { buf, vers } => {
                let mut r = ToDecompress::new(Cursor::new(buf));
                r.extract(path)?;
                vers
            }
            Self::File { dir, vers } => {
                let mut r = ToDecompress::new(BufReader::new(std::fs::File::open(dir)?));
                r.extract(path)?;
                vers
            }
        };
        if rename {
            std::fs::rename(par.join("go"), par.join(&format!("go{}", vers.version)))
                .with_context(|| "Rename error".to_string())
        } else {
            Ok(())
        }
    }
}

impl GoVersions {
    #[instrument(err, ret)]
    pub fn new(path: PathBuf) -> Result<Self> {
        if path.exists() {
            return Self::from_file(&path);
        }
        let rels: Vec<GoVersion> = CLIENT
            .get(DLURL)
            .send()?
            .json()
            .context("Failed to deserialize")?;
        let mut parsed: Vec<GoVersion> = rels
            .into_par_iter()
            .filter_map(|x| x.parse().ok())
            .collect();
        parsed.par_sort_unstable_by(|a, b| a.parsed.cmp(&b.parsed));
        let latest = parsed.get(0).context("No latest found")?;
        Ok(Self {
            latest: latest.clone(),
            path,
            versions: parsed,
        })
    }
    #[instrument(err, ret(Display))]
    pub(crate) fn from_file(path: &Path) -> Result<Self> {
        let read = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(read.as_str())?)
    }
    #[instrument(err)]
    fn save(&self, list_path: &Path) -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(list_path)?;
        let to_write = toml::to_string_pretty(self)?;
        file.write_all(to_write.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }
    pub fn latest(&self) -> GoVersion {
        self.latest.clone()
    }

    pub fn chosen_version(&self, vers: Version) -> Result<GoVersion> {
        let res = self
            .versions
            .par_iter()
            .find_any(|x| x.parsed == vers)
            .context("No version found")?;
        Ok(res.clone())
    }
}

impl GoVersion {
    /// Downloads the required version
    pub fn download(&self, output: Option<PathBuf>, workers: u8) -> Result<Downloaded> {
        let style = manic::ProgressStyle::default_bar()
            .template("{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-");
        let f = self.wanted_file()?;
        let mut client = Downloader::new(f.get_url().as_str(), workers)?;
        client.progress_bar();
        let hash = manic::Hash::new_sha256(f.sha256.to_string());
        client.verify(hash);
        client.bar_style(style);
        if let Some(path) = output {
            let path_str = path.to_str().context("No path")?;
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
    pub fn wanted_file(&self) -> Result<&File> {
        self.files
            .par_iter()
            .find_any(|x| x.os == OS && x.arch == ARCH.as_str() && x.kind == KIND)
            .context("No file found")
    }
    #[instrument(err, ret)]
    pub(crate) fn check_newer(&self, path: &Path) -> Result<bool> {
        if let Some(s) = get_local_version(path)? {
            if s < self.parsed {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(true)
        }
    }
    #[instrument(err, ret)]
    pub fn parse(mut self) -> Result<Self> {
        if self.is_parsed {
            Ok(self)
        } else {
            self.parsed = Version::parse(
                &self
                    .version
                    .replace("go", "")
                    .replace("rc", "-rc.")
                    .replace("beta", "-beta."),
            )?;
            self.is_parsed = true;
            Ok(self)
        }
    }
}

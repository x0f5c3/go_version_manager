use crate::GoVersions;
use directories::{BaseDirs, ProjectDirs};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
// use crate::error::Result;
use anyhow::{Context, Result};
use reqwest::Proxy;
use semver::Version;

pub const DL_URL: &str = "https://go.dev/dl";

#[cfg(target_os = "windows")]
pub const PATH_SEPERATOR: &str = ";";

#[cfg(not(target_os = "windows"))]
pub const PATH_SEPERATOR: &str = ":";

#[derive(Debug, Deserialize, Serialize)]
struct SysConfig {
    file_ext: String,
    config_dir: PathBuf,
    proxies: Option<Vec<String>>,
    #[serde(skip)]
    client: manic::Client,
    versions_list: PathBuf,
    install_dir: PathBuf,
    envs_dir: PathBuf,
    current: Option<Version>,
}

impl SysConfig {
    pub fn default() -> Result<Self> {
        let base_dirs =
            ProjectDirs::from("rs", "", "go_version_manager").context("Can't get project dirs")?;
        let config_dirs = base_dirs.config_dir().to_path_buf().join("go_manager");
        let config_path = config_dirs.join("go_manager.toml");
        let def_install = BaseDirs::new()
            .map(|x| x.home_dir().join(".goenvs"))
            .context("Can't get base dirs")?;
        if config_path.exists() {
            Self::from_path(config_path)
        } else {
            let (install_dir, version): (PathBuf, Option<Version>) = which::which("go")
                .ok()
                .and_then(|x| {
                    let vers = duct::cmd!(&x, "version")
                        .read()
                        .ok()?
                        .split(' ')
                        .nth(2)?
                        .replace("go", "");
                    Some((
                        x.parent().map(|x| x.to_path_buf())?,
                        Version::parse(&vers).ok(),
                    ))
                })
                .and_then(|(x, y)| Some((x.parent().map(|x| x.to_path_buf())?, y)))
                .unwrap_or_else(|| (def_install, None));
            let versions_list = config_dirs.join("versions.toml");
            let ret = SysConfig {
                file_ext: FILE_EXT.clone(),
                config_dir: config_dirs,
                proxies: None,
                client: manic::Client::new(),
                versions_list,
                install_dir,
                envs_dir: base_dirs.data_local_dir().join("go_envs"),
                current: version,
            };
            std::fs::write(config_path, toml::to_string_pretty(&ret)?)?;
            Ok(ret)
        }
    }
    pub fn from_path(file: PathBuf) -> Result<Self> {
        let mut ret: SysConfig = toml::from_str(&std::fs::read_to_string(file)?)?;
        if let Some(p) = &ret.proxies {
            let mut client = {
                let mut b = manic::Client::builder();
                for i in p {
                    b = b.proxy(Proxy::all(i)?);
                }
                b.build()?
            };
            ret.client = client;
        } else {
            ret.client = manic::Client::new();
        }
        Ok(ret)
    }
    pub fn save(&self) -> Result<()> {
        std::fs::write(
            self.config_dir.join("go_manager.toml"),
            toml::to_string_pretty(&self)?,
        )
        .context("Failed to save the config")
    }
}

lazy_static! {
    pub static ref FILE_EXT: String = {
        let os = match std::env::consts::OS {
            "windows" => "windows",
            "macos" => "darwin",
            "linux" => "linux",
            "freebsd" => "freebsd",
            x => panic!("OS {} not supported", x),
        };
        let arch = match std::env::consts::ARCH {
            "x86_64" => "amd64",
            "x86" => "386",
            "aarch64" => "arm64",
            "arm" => "armv6l",
            "powerpc64" => "ppc64le",
            "s390x" => "s390x",
            x => panic!("ARCH {} not supported", x),
        };
        let ext = match os {
            "windows" => "zip",
            _ => "tar.gz",
        };
        format!("{}-{}.{}", os, arch, ext)
    };
    pub static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from("rs", "", "Go Manager").unwrap();
    pub static ref CONFIG_DIR: PathBuf = {
        let res = PROJECT_DIRS.config_dir().to_path_buf();
        if !res.exists() {
            std::fs::create_dir_all(&res).unwrap();
        }
        res
    };
    pub static ref CLIENT: manic::Client = manic::Client::new();
    pub static ref DL_PAGE: String = CLIENT.get(DL_URL).send().unwrap().text().unwrap();
    pub static ref CONFIG_PATH: PathBuf = CONFIG_DIR.join("config.toml");
    pub static ref VERSION_LIST: PathBuf = CONFIG_DIR.join("versions.json");
    pub static ref DEFAULT_INSTALL: PathBuf = {
        if cfg!(windows) {
            PathBuf::from("C:\\Go")
        } else {
            PathBuf::from("/usr/local/go")
        }
    };
    pub static ref CURRENT_INSTALL: Option<PathBuf> = which::which("go").ok();
    pub static ref ENVS_DIR: PathBuf = PROJECT_DIRS.data_local_dir().join("envs");
    pub static ref GIT_VERSIONS: Vec<Version> = {
        let output = GoVersions::raw_git_versions().unwrap();
        GoVersions::parse_versions(output).unwrap()
    };
}

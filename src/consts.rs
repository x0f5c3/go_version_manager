use crate::utils::get_local_path;
use crate::GoVersions;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
// use crate::error::Result;
use anyhow::Result;
use versions::SemVer;

pub const DL_URL: &str = "https://go.dev/dl";

#[cfg(target_os = "windows")]
pub const PATH_SEPERATOR: &str = ";";

#[cfg(not(target_os = "windows"))]
pub const PATH_SEPERATOR: &str = ":";

#[derive(Debug, Deserialize, Serialize)]
struct SysConfig {
    file_ext: String,
    config_dir: PathBuf,
    proxies: Option<String>,
    #[serde(skip)]
    client: manic::Client,
    versions_list: PathBuf,
    install_dir: PathBuf,
    current_install: Option<PathBuf>,
    envs_dir: PathBuf,
}
impl SysConfig {
    fn default() -> Result<Self> {
        let config_dirs = directories::BaseDirs::new()
            .unwrap()
            .config_dir()
            .to_path_buf();
        let config_path = config_dirs.join("go_manager.json");
        if config_path.exists() {
            let mut ret: SysConfig = toml::from_str(&std::fs::read_to_string(config_path)?)?;
            if let Some(p) = &ret.proxies {
                let client = manic::Client::builder()
                    .proxy(reqwest::Proxy::http(p)?)
                    .proxy(reqwest::Proxy::https(p)?)
                    .build()
                    .unwrap();
                ret.client = client;
            } else {
                ret.client = manic::Client::new();
            }
            Ok(ret)
        } else {
            let ret = SysConfig {
                file_ext: FILE_EXT.clone(),
                config_dir: CONFIG_DIR.clone(),
                proxies: None,
                client: manic::Client::new(),
                versions_list: VERSION_LIST.clone(),
                install_dir: DEFAULT_INSTALL.clone(),
                current_install: CURRENT_INSTALL.clone(),
                envs_dir: ENVS_DIR.clone(),
            };
            std::fs::write(config_path, toml::to_string_pretty(&ret)?)?;
            Ok(ret)
        }
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
    pub static ref CONFIG_PATH: PathBuf = CONFIG_DIR.join("config.json");
    pub static ref VERSION_LIST: PathBuf = CONFIG_DIR.join("versions.json");
    pub static ref DEFAULT_INSTALL: PathBuf = {
        if cfg!(windows) {
            PathBuf::from("C:\\Go")
        } else {
            PathBuf::from("/usr/local/go")
        }
    };
    pub static ref CURRENT_INSTALL: Option<PathBuf> = get_local_path().unwrap_or(None);
    pub static ref ENVS_DIR: PathBuf = PROJECT_DIRS.data_local_dir().join("envs");
    pub static ref GIT_VERSIONS: Vec<SemVer> = {
        let output = GoVersions::raw_git_versions().unwrap();
        GoVersions::parse_versions(output).unwrap()
    };
}

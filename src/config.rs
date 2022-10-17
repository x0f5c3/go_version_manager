use crate::consts::VERSION_LIST;
use crate::goversion::GoVersion;
use crate::utils::get_local_version;
use crate::GoVersions;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub(crate) struct App {
    pub(crate) config: Config,
    pub(crate) versions: GoVersions,
}

impl App {
    pub(crate) fn new(config: Config, list_path: PathBuf) -> Result<Self> {
        let versions = if list_path.exists() {
            GoVersions::from_file(&list_path)?
        } else {
            GoVersions::new(VERSION_LIST.clone())?
        };
        Ok(Self { config, versions })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Config {
    pub(crate) install_path: PathBuf,
    pub(crate) list_path: PathBuf,
    pub(crate) config_path: PathBuf,
    #[serde(skip)]
    pub(crate) list: Option<GoVersions>,
    pub(crate) current: Option<GoVersion>,
}

impl Config {
    fn from_file(path: PathBuf) -> Result<Self> {
        let conf = fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&conf)?)
    }
    pub fn new(install_path: PathBuf, config_path: PathBuf) -> Result<Self> {
        if config_path.exists() {
            return Self::from_file(config_path);
        }
        let vers = get_local_version(&install_path)?;
        let list_path = config_path
            .parent()
            .map(|x| x.join("versions.toml"))
            .unwrap_or_else(|| VERSION_LIST.clone());
        let list = GoVersions::new(list_path.clone()).ok();
        let govers = if let Some(v) = vers {
            if let Some(list) = &list {
                list.chosen_version(v).ok()
            } else {
                None
            }
        } else {
            None
        };
        Ok(Self {
            install_path,
            list_path,
            config_path,
            list,
            current: govers,
        })
    }
    pub fn save(&self) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(&self.config_path)?;
        let to_write = serde_json::to_string_pretty(&self)?;
        file.write_all(to_write.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }
}

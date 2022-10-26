use crate::consts::VERSION_LIST;
use crate::goversion::GoVersion;
use crate::utils::get_local_version;
use crate::GoVersions;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct App {
    pub(crate) config: Config,
    #[serde(skip)]
    pub(crate) versions: GoVersions,
}

impl App {
    pub(crate) fn new_from_list(config: Config, list_path: PathBuf) -> Result<Self> {
        let versions = GoVersions::new(list_path)?;
        Ok(Self { config, versions })
    }
    pub(crate) fn new(config: Config) -> Result<Self> {
        let versions = if let Some(l) = &config.list {
            l.clone()
        } else {
            GoVersions::new(config.list_path.clone()).context("No list available")?
        };
        // .as_ref()
        // .and_then(|x| Some(x.clone()))
        // .or_else(|| GoVersions::new(config.list_path.clone()).ok())
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
        Ok(toml::from_str(&conf)?)
    }
    pub fn into_app(self) -> Result<App> {
        App::new(self)
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
        if let Some(l) = &self.list {
            let mut file = fs::OpenOptions::new()
                .truncate(true)
                .create(true)
                .write(true)
                .open(&self.list_path)?;
            file.write_all(toml::to_string_pretty(l)?.as_bytes())?;
        }
        let mut file = fs::OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(&self.config_path)?;
        let to_write = toml::to_string_pretty(&self)?;
        file.write_all(to_write.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }
}

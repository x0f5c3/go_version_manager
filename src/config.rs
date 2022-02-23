use crate::consts::VERSION_LIST;
use crate::goversion::GoVersion;
use crate::utils::get_local_version;
use crate::GoVersions;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Config {
    pub(crate) install_path: PathBuf,
    pub(crate) list_path: PathBuf,
    pub(crate) config_path: PathBuf,
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
        let govers = if let Some(v) = vers {
            Some(GoVersions::download_chosen(v)?)
        } else {
            None
        };
        let list_path = config_path
            .parent()
            .map(|x| x.join("versions.toml"))
            .unwrap_or_else(|| VERSION_LIST.clone());
        Ok(Self {
            install_path,
            list_path,
            config_path,
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

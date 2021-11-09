use crate::consts::{CONFIG_PATH, DEFAULT_INSTALL, VERSION_LIST};
use crate::goversion::{get_local_version, GoVersion};
use crate::{Error, GoVersions, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Config {
    install_path: PathBuf,
    config_path: PathBuf,
    current: Option<GoVersion>,
}

impl Config {
    fn from_file(path: PathBuf) -> Result<Self> {
        let conf = fs::read_to_string(&path)?;
        serde_json::from_str(&conf).map_err(Error::JSONErr)
    }
    pub fn new(install_path: PathBuf, config_path: Option<PathBuf>) -> Result<Self> {
        let new_path: PathBuf;
        let list_path: PathBuf;
        if let Some(p) = config_path {
            if p.exists() {
                return Self::from_file(p);
            }
            new_path = p;
            list_path = new_path
                .parent()
                .ok_or(Error::PathBufErr)?
                .join("versions.toml")
        } else {
            if CONFIG_PATH.exists() {
                return Self::from_file(CONFIG_PATH.clone());
            }
            new_path = CONFIG_PATH.clone();
            list_path = VERSION_LIST.clone();
        }
        let vers = get_local_version(&DEFAULT_INSTALL)?;
        let govers = if let Some(v) = vers {
            Some(GoVersions::new(Some(&list_path))?.chosen_version(v)?)
        } else {
            None
        };
        Ok(Self {
            install_path,
            config_path: new_path,
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

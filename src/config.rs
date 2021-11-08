use std::path::PathBuf;
use crate::goversion::{get_local_version, GoVersion};
use crate::{Error, GoVersions, Result};
use serde::{Deserialize, Serialize};
use crate::consts::CONFIG_PATH;
use tokio::fs::read_to_string;



#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Config {
    install_path: PathBuf,
    config_path: PathBuf,
    current: GoVersion,
}


impl Config {
    async fn from_file(path: PathBuf) -> Result<Self> {
        let conf = read_to_string(&path).await?;
        serde_json::from_str(&conf).map_err(|e| Error::JSONErr(e))
    }
    pub async fn new(install_path: PathBuf, config_path: Option<PathBuf>, git: bool) -> Result<Self> {
        let new_path: PathBuf;
        if let Some(p) = config_path {
            if p.exists() {
                return Self::from_file(p).await
            }
            new_path = p.clone();
        } else {
            new_path = CONFIG_PATH.clone();
        }
        let vers = get_local_version()?.ok_or(Error::NoVersion)?;
        let govers = GoVersions::new(git).await?.chosen_version(vers)?;
        Ok(Self{
            install_path,
            config_path: new_path,
            current: govers
        })
    }
}



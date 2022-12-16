use std::fs;
use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::consts::{CONFIG_PATH, CURRENT_INSTALL, DEFAULT_INSTALL};
// use crate::Result;
use crate::goversion::{GoVersion, GoVersions};
use anyhow::Result;
use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;

/// Initialize the config
#[derive(Debug, Clone, Args)]
pub(crate) struct Init {
    #[clap(short, long)]
    config_path: Option<PathBuf>,
    install_path: Option<PathBuf>,
}

impl Init {
    pub(crate) fn run(self) -> Result<()> {
        let install_path = self
            .install_path
            .or_else(|| CURRENT_INSTALL.clone())
            .or_else(|| Some(DEFAULT_INSTALL.clone()))
            .unwrap();
        if CURRENT_INSTALL.is_some() {
            paris::info!("Found local install, will be using its path and version");
        }
        let config_path = self.config_path.unwrap_or_else(|| CONFIG_PATH.clone());
        let c = Config::new(install_path, config_path)?;
        c.save()?;
        paris::info!("Config path: {}", c.config_path.display());
        paris::info!("Install path: {}", c.install_path.display());
        if let Some(v) = c.current {
            paris::info!("Current version: {}", v.version);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Subcommand)]
enum ConfigSubCommands {
    Init(Config),
    Set,
    Get,
    List,
}

#[derive(Debug, Deserialize, Serialize, Clone, Args)]
pub(crate) struct Config {
    #[arg(short, long)]
    pub(crate) install_path: PathBuf,
    #[arg(short, long)]
    pub(crate) list_path: PathBuf,
    #[arg(short, long)]
    pub(crate) config_path: PathBuf,
    #[serde(skip)]
    #[arg(skip)]
    pub(crate) list: Option<GoVersions>,
    #[arg(skip)]
    pub(crate) current: Option<GoVersion>,
}

impl Config {
    fn figment(self) -> Result<Self> {
        let conf: Config = Figment::new()
            .merge(Serialized::defaults(self))
            .merge(Toml::file(CONFIG_PATH.as_path()))
            .merge(Env::prefixed("GOM_"))
            .extract()?;
        Ok(conf)
    }
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

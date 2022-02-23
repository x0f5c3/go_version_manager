use std::path::PathBuf;

use structopt::StructOpt;

use crate::config::Config;
use crate::consts::{CONFIG_PATH, CURRENT_INSTALL, DEFAULT_INSTALL};
// use crate::Result;
use anyhow::Result;

/// Initialize the config
#[derive(Debug, Clone, StructOpt)]
pub(crate) struct Init {
    #[structopt(short, long, parse(from_os_str))]
    config_path: Option<PathBuf>,
    #[structopt(parse(from_os_str))]
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

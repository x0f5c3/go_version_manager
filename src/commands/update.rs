use std::path::PathBuf;

use structopt::StructOpt;

use crate::commands::utils::check_writable;
use crate::config::Config;
use crate::consts::{CONFIG_PATH, CURRENT_INSTALL, DEFAULT_INSTALL};
use crate::error::Error;
use crate::goversion::GoVersions;
// use crate::Result;
use anyhow::{Context, Result};

/// Update the existing instalation
#[derive(Debug, Clone, StructOpt)]
pub(crate) struct Update {
    #[structopt(short, long)]
    workers: Option<u8>,
    #[structopt(short, long, parse(from_os_str))]
    config_path: Option<PathBuf>,
    #[structopt(short, long, parse(from_os_str))]
    install_path: Option<PathBuf>,
}

impl Update {
    pub(crate) fn run(self) -> Result<()> {
        let workers = self.workers.unwrap_or(num_cpus::get() as u8);
        let config_path = self.config_path.unwrap_or_else(|| CONFIG_PATH.clone());
        let install_path = self
            .install_path
            .or_else(|| CURRENT_INSTALL.clone())
            .unwrap_or_else(|| DEFAULT_INSTALL.clone());
        let check = check_writable(
            install_path
                .parent()
                .context("Failed to get the parent directory")?,
        )?;
        let c = Config::new(install_path.clone(), config_path)?;
        let latest = GoVersions::download_latest()?;
        let res = latest.check_newer(&c.install_path).and_then(|x| {
            if x {
                if !check {
                    paris::error!(
                        "Cannot update go, you don't have write access to {}",
                        install_path.display()
                    );
                    quit::with_code(1);
                }
                latest.download(None, workers)
            } else {
                paris::success!("You already have the latest version");
                quit::with_code(0);
            }
        });
        if check_writable(c.install_path.parent().ok_or(Error::PathBufErr)?)? {
            res?.unpack(&install_path, false)
        } else {
            Err(Error::NOPerm.into())
        }
    }
}

use std::path::PathBuf;

use clap::Parser;

use crate::config::Config;
use crate::consts::{CONFIG_PATH, CURRENT_INSTALL, DEFAULT_INSTALL};
use crate::goversion::GoVersions;
use crate::utils::check_writable;
use anyhow::{anyhow, Context, Result};

/// Update the existing instalation
#[derive(Debug, Clone, Parser)]
pub(crate) struct Update {
    #[clap(short, long)]
    workers: Option<u8>,
    #[clap(short, long)]
    config_path: Option<PathBuf>,
    #[clap(short, long)]
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
        let latest = GoVersions::new(c.list_path.clone())?.latest();
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
        if check_writable(c.install_path.parent().context("No parent")?)? {
            res?.unpack(&install_path, false)
        } else {
            Err(anyhow!("{} is not writable", c.install_path.display()))
        }
    }
}

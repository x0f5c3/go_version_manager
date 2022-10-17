use std::path::PathBuf;

use clap::Parser;
use dialoguer::console::Term;
use semver::Version;

use crate::ask_for_version;
use crate::config::Config;
use crate::consts::{CONFIG_PATH, DEFAULT_INSTALL};
use crate::goversion::{GoVersion, GoVersions};
use crate::utils::{check_in_path, check_writable};
use anyhow::{anyhow, Context, Result};

/// Install the chosen or latest golang version
#[derive(Debug, Clone, Parser)]
pub(crate) struct Install {
    #[clap(short, long)]
    config_path: Option<PathBuf>,
    #[clap(conflicts_with("config_path"))]
    install_path: Option<PathBuf>,
    #[clap(short, long)]
    workers: Option<u8>,
    #[clap(long, conflicts_with("interactive"))]
    version: Option<Version>,
    #[clap(short, long)]
    interactive: bool,
}

impl Install {
    pub(crate) fn run(self) -> Result<()> {
        let install_path = self.install_path.unwrap_or_else(|| DEFAULT_INSTALL.clone());
        let config_path = self.config_path.unwrap_or_else(|| CONFIG_PATH.clone());
        let workers = self.workers.unwrap_or_else(|| num_cpus::get() as u8);
        let c = Config::new(install_path, config_path)?;
        let versions = GoVersions::new(c.list_path.clone())?;
        let golang = {
            if let Some(vers) = self.version {
                let chosen: GoVersion = versions.chosen_version(vers)?;
                chosen
            } else if self.interactive {
                let term = Term::stdout();
                let vers = ask_for_version(&term, &versions)?;
                let chosen: GoVersion = versions.chosen_version(vers.parsed)?;
                chosen
            } else {
                versions.latest()
            }
        };
        if check_writable(c.install_path.parent().context("No parent")?)? {
            let res = golang.download(None, workers)?;
            res.unpack(&c.install_path, false)?;
            let bin_path = &c.install_path.join("bin");
            if !check_in_path(bin_path)? {
                paris::info!("Directory {} not in PATH", bin_path.display());
            }
            Ok(())
        } else {
            Err(anyhow!("{} is not writable", c.install_path.display()))
        }
    }
}

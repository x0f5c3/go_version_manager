use std::path::PathBuf;

use clap::Parser;
use dialoguer::console::Term;
use semver::Version;

use crate::goversion::GoVersions;
// use crate::Result;
use crate::consts::VERSION_LIST;
use crate::{ask_for_version, Downloaded};
use anyhow::{Context, Result};

/// Download golang version to file
#[derive(Debug, Clone, Parser)]
pub(crate) struct Download {
    output: PathBuf,
    #[clap(short, long)]
    workers: Option<u8>,
    #[clap(long, conflicts_with("interactive"))]
    version: Option<Version>,
    #[clap(short, long)]
    interactive: bool,
}

impl Download {
    pub(crate) fn run(self) -> Result<()> {
        let workers = self.workers.unwrap_or(num_cpus::get() as u8);
        let term = Term::stdout();
        let versions = GoVersions::new(VERSION_LIST.clone())?;
        let golang = {
            if let Some(vers) = self.version {
                let chosen: crate::goversion::GoVersion = versions.chosen_version(vers)?;
                chosen
            } else if self.interactive {
                let vers = ask_for_version(&term, &versions)?;
                let chosen: crate::goversion::GoVersion = versions.chosen_version(vers.parsed)?;
                chosen
            } else {
                versions.latest()
            }
        };
        paris::info!(
            "<b><blue>Downloading golang version {}</></b>",
            &golang.version
        );
        let file_path = golang.download(Some(self.output), workers)?;
        if let Downloaded::File { dir, vers: _ } = file_path {
            let path_str = dir.to_str().context("Path cannot be converted to string")?;
            paris::success!(
                "<b><bright green>Golang has been downloaded to {}</></b>",
                path_str
            );
        }
        Ok(())
    }
}

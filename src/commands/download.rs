use std::path::PathBuf;

use dialoguer::console::Term;
use structopt::StructOpt;
use versions::SemVer;

use crate::consts::FILE_EXT;
use crate::error::Error;
use crate::goversion::{GoVersion, GoVersions};
// use crate::Result;
use crate::{ask_for_version, Downloaded};
use anyhow::Result;

use super::utils::parse_version;

/// Download golang version to file
#[derive(Debug, Clone, StructOpt)]
pub(crate) struct Download {
    #[structopt(parse(from_os_str))]
    output: PathBuf,
    #[structopt(short, long)]
    workers: Option<u8>,
    #[structopt(long, parse(try_from_str = parse_version), conflicts_with("interactive"))]
    version: Option<SemVer>,
    #[structopt(short, long)]
    interactive: bool,
}

impl Download {
    pub(crate) fn run(self) -> Result<()> {
        let workers = self.workers.unwrap_or(num_cpus::get() as u8);
        let term = Term::stdout();
        let versions = GoVersions::new(None)?;
        let golang = {
            if let Some(vers) = self.version {
                let chosen: GoVersion = versions.chosen_version(vers)?;
                chosen
            } else if self.interactive {
                let vers = ask_for_version(&term, &versions)?;
                let chosen: GoVersion = versions.chosen_version(vers)?;
                chosen
            } else {
                versions.latest()
            }
        };
        paris::info!(
            "<b><bright blue>Filename: go{}.{}</></b>",
            golang.version,
            FILE_EXT.as_str()
        );
        paris::info!(
            "<b><blue>Downloading golang version {}</></b>",
            &golang.version
        );
        let file_path = golang.download(Some(self.output), workers)?;
        if let Downloaded::File { dir, vers: _ } = file_path {
            let path_str = dir.to_str().ok_or(Error::PathBufErr)?;
            paris::success!(
                "<b><bright green>Golang has been downloaded to {}</></b>",
                path_str
            );
        }
        Ok(())
    }
}

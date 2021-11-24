use std::io::Cursor;
use std::path::PathBuf;

use structopt::StructOpt;

use crate::commands::utils::check_writable;
use crate::config::Config;
use crate::consts::{CONFIG_PATH, DEFAULT_INSTALL};
use crate::decompressor::ToDecompress;
use crate::Downloaded;
use crate::error::Error;
use crate::goversion::GoVersions;
use crate::Result;

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
		let install_path = self.install_path.unwrap_or_else(|| DEFAULT_INSTALL.clone());
		let c = Config::new(install_path, config_path)?;
		let latest = GoVersions::download_latest()?;
		let res = {
			if let Some(v) = c.current {
				if v.version == latest.version {
					paris::success!("You already have the latest version {}", v.version);
					quit::with_code(0);
				} else {
					let res = std::fs::metadata(&c.install_path);
					if let Err(e) = res {
						if let std::io::ErrorKind::PermissionDenied = e.kind() {
							paris::error!(
                                "You don't have privs to install in {}",
                                c.install_path.display()
                            );
							quit::with_code(127);
						} else {
							Err(e.into())
						}
					} else {
						latest.download(None, workers)
					}
				}
			} else if check_writable(c.install_path.parent().ok_or(Error::PathBufErr)?)? {
				latest.download(None, workers)
			} else {
				Err(Error::PathBufErr)
			}
		};
		if let Ok(Downloaded::Mem(m)) = res {
			let mut dec = ToDecompress::new(Cursor::new(m))?;
			dec.extract(&c.install_path)
		} else {
			Err(Error::NoVersion)
		}
	}
}

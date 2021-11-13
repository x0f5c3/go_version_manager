use std::io::Cursor;
use crate::config::Config;
use crate::consts::{CONFIG_PATH, DEFAULT_INSTALL, FILE_EXT};
use crate::error::Error;
use crate::goversion::{GoVersion, GoVersions};
use crate::Result;
use crate::{ask_for_version, Downloaded};
use dialoguer::console::Term;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use versions::SemVer;
use crate::decompressor::ToDecompress;

#[derive(Debug, StructOpt)]
/// I will download Go language install and check its hash to verify I did it correctly{n}Keep calm and carry on
pub(crate) enum Command {
    Init(Init),
    Update(Update),
    Install(Install),
    Download(Download),
    Completetions(Completetions),
}

impl Command {
    pub fn run(self) -> Result<()> {
        match self {
            Self::Download(d) => {
                let workers = d.workers.unwrap_or(num_cpus::get() as u8);
                let term = Term::stdout();
                let versions = GoVersions::new(None)?;
                let golang = {
                    if let Some(vers) = d.version {
                        let chosen: GoVersion = versions.chosen_version(vers)?;
                        chosen
                    } else if d.interactive {
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
                let file_path = golang.download(Some(d.output), workers)?;
                if let Downloaded::File(path) = file_path {
                    let path_str = path.to_str().ok_or(Error::PathBufErr)?;
                    paris::success!(
                        "<b><bright green>Golang has been downloaded to {}</></b>",
                        path_str
                    );
                }
                Ok(())
            }
            Self::Init(i) => {
                let install_path = i.install_path.unwrap_or_else(|| DEFAULT_INSTALL.clone());
                let config_path = i.config_path.unwrap_or_else(|| CONFIG_PATH.clone());
                let c = Config::new(install_path, config_path)?;
                c.save()
            },
            Self::Update(u) => {
                let workers = u.workers.unwrap_or(num_cpus::get() as u8);
                let config_path = u.config_path.unwrap_or_else(|| CONFIG_PATH.clone());
                let install_path = u.install_path.unwrap_or_else(|| DEFAULT_INSTALL.clone());
                let c = Config::new(install_path, config_path)?;
                let latest = GoVersions::download_latest()?;
                let res = {
                    if let Some(v) = c.current {
                        if v.version == latest.version {
                            paris::success!("You already have the latest version {}", v.version);
                            quit::with_code(0);
                        } else {
                            let res = std::fs::metadata(&install_path);
                            if let Err(e) = res {
                                if let std::io::ErrorKind::PermissionDenied = e.kind() {
                                    paris::error!("You don't have privs to install in {}", install_path.display());
                                    quit::with_code(127);
                                } else {
                                    Err(e.into())
                                }
                            } else {
                                latest.download(None, workers)
                            }
                        }
                    } else {
                        if check_writable(&install_path) {
                            latest.download(None, workers)
                        } else {
                            Err(Error::PathBufErr)
                        }
                    }
                    };
                if let Ok(Downloaded::Mem(m)) = res {
                    let mut dec = ToDecompress::new(Cursor::new(m))?;
                    dec.extract(&install_path.parent().ok_or(Error::PathBufErr)?)
                } else {
                    Err(Error::NoVersion)
                }
                },
            _ => unimplemented!("Unimplemented"),
        }
    }
}


fn check_writable(p: &Path) -> bool {
    std::fs::metadata(p).is_ok()
}
    /// Initialize the config
    #[derive(Debug, Clone, StructOpt)]
    struct Init {
        #[structopt(short, long, parse(from_os_str))]
        config_path: Option<PathBuf>,
#[structopt(parse(from_os_str))]
install_path: Option<PathBuf>,
}
/// Update the existing instalation
#[derive(Debug, Clone, StructOpt)]
struct Update {
#[structopt(short, long)]
workers: Option<u8>,
#[structopt(short, long, parse(from_os_str))]
config_path: Option<PathBuf>,
#[structopt(short, long, parse(from_os_str))]
install_path: Option<PathBuf>,
}
/// Install the chosen or latest golang version
#[derive(Debug, Clone, StructOpt)]
struct Install {
#[structopt(short, long, parse(from_os_str))]
config_path: Option<PathBuf>,
#[structopt(parse(from_os_str), conflicts_with("config_path"))]
install_path: Option<PathBuf>,
#[structopt(short, long)]
workers: Option<u8>,
#[structopt(long, parse(try_from_str = parse_version), conflicts_with("interactive"))]
version: Option<SemVer>,
#[structopt(short, long)]
interactive: bool,
}
/// Download golang version to file
#[derive(Debug, Clone, StructOpt)]
struct Download {
#[structopt(parse(from_os_str))]
output: PathBuf,
#[structopt(short, long)]
workers: Option<u8>,
#[structopt(long, parse(try_from_str = parse_version), conflicts_with("interactive"))]
version: Option<SemVer>,
#[structopt(short, long)]
interactive: bool,
}
/// Generate completions
#[derive(Debug, Clone, StructOpt)]
struct Completetions {
shell: structopt::clap::Shell,
#[structopt(parse(from_os_str))]
out_dir: PathBuf,
}

fn parse_version(src: &str) -> Result<SemVer> {
    SemVer::new(src).ok_or(Error::VersParse)
}

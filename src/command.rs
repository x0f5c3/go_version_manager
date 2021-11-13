use crate::config::Config;
use crate::consts::{CONFIG_PATH, DEFAULT_INSTALL, FILE_EXT};
use crate::decompressor::ToDecompress;
use crate::error::Error;
use crate::goversion::{GoVersion, GoVersions};
use crate::Result;
use crate::{ask_for_version, Downloaded};
use dialoguer::console::Term;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use versions::SemVer;

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
            Self::Download(d) => d.run(),
            Self::Init(i) => i.run(),
            Self::Update(u) => u.run(),
            Self::Completetions(c) => c.run(),
            Self::Install(i) => i.run(),
        }
    }
}

fn check_writable(p: &Path) -> bool {
    let res = std::fs::write(p.join("test"), "test");
    if res.is_ok() {
        std::fs::remove_file(p.join("test")).is_ok()
    } else {
        false
    }
}
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
        let install_path = self.install_path.unwrap_or_else(|| DEFAULT_INSTALL.clone());
        let config_path = self.config_path.unwrap_or_else(|| CONFIG_PATH.clone());
        let c = Config::new(install_path, config_path)?;
        c.save()
    }
}

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
        let c = Config::new(install_path.clone(), config_path)?;
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
                            paris::error!(
                                "You don't have privs to install in {}",
                                install_path.display()
                            );
                            quit::with_code(127);
                        } else {
                            Err(e.into())
                        }
                    } else {
                        latest.download(None, workers)
                    }
                }
            } else if check_writable(&install_path) {
                latest.download(None, workers)
            } else {
                Err(Error::PathBufErr)
            }
        };
        if let Ok(Downloaded::Mem(m)) = res {
            let mut dec = ToDecompress::new(Cursor::new(m))?;
            dec.extract(install_path.parent().ok_or(Error::PathBufErr)?)
        } else {
            Err(Error::NoVersion)
        }
    }
}

/// Install the chosen or latest golang version
#[derive(Debug, Clone, StructOpt)]
pub(crate) struct Install {
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

impl Install {
    pub(crate) fn run(self) -> Result<()> {
        let install_path = self.install_path.unwrap_or_else(|| DEFAULT_INSTALL.clone());
        let config_path = self.config_path.unwrap_or_else(|| CONFIG_PATH.clone());
        let workers = self.workers.unwrap_or_else(|| num_cpus::get() as u8);
        let c = Config::new(install_path.clone(), config_path)?;
        let versions = GoVersions::new(Some(&c.list_path))?;
        let golang = {
            if let Some(vers) = self.version {
                let chosen: GoVersion = versions.chosen_version(vers)?;
                chosen
            } else if self.interactive {
                let term = Term::stdout();
                let vers = ask_for_version(&term, &versions)?;
                let chosen: GoVersion = versions.chosen_version(vers)?;
                chosen
            } else {
                versions.latest()
            }
        };
        if check_writable(&install_path) {
            let res = golang.download(None, workers)?;
            if let Downloaded::Mem(v) = res {
                let mut dec = ToDecompress::new(Cursor::new(v))?;
                dec.extract(&install_path)
            } else {
                Err(Error::PathBufErr)
            }
        } else {
            Err(Error::PathBufErr)
        }
    }
}
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
        if let Downloaded::File(path) = file_path {
            let path_str = path.to_str().ok_or(Error::PathBufErr)?;
            paris::success!(
                "<b><bright green>Golang has been downloaded to {}</></b>",
                path_str
            );
        }
        Ok(())
    }
}

/// Generate completions
#[derive(Debug, Clone, StructOpt)]
pub(crate) struct Completetions {
    shell: structopt::clap::Shell,
    #[structopt(parse(from_os_str))]
    out_dir: PathBuf,
}

impl Completetions {
    pub(crate) fn run(self) -> Result<()> {
        let mut app: structopt::clap::App = Self::clap();
        app.gen_completions("go_version_manager", self.shell, self.out_dir);
        Ok(())
    }
}

fn parse_version(src: &str) -> Result<SemVer> {
    SemVer::new(src).ok_or(Error::VersParse)
}

use crate::ask_for_version;
use crate::consts::FILE_EXT;
use crate::error::Error;
use crate::goversion::{GoVersion, GoVersions};
use crate::Result;
use dialoguer::console::Term;
use std::path::PathBuf;
use structopt::StructOpt;
use versions::SemVer;
#[derive(Debug, StructOpt)]
pub(crate) struct Opt {
    #[structopt(parse(from_os_str))]
    pub output: PathBuf,
    #[structopt(short, long, default_value = "2")]
    pub workers: u8,
    #[structopt(short, long, conflicts_with("version"), conflicts_with("interactive"))]
    pub update: bool,
    #[structopt(long, parse(try_from_str = parse_version), conflicts_with("interactive"))]
    pub version: Option<SemVer>,
    #[structopt(short, long)]
    pub interactive: bool,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
pub(crate) enum Command {
    Init {
        #[structopt(short, long, parse(from_os_str))]
        config_path: Option<PathBuf>,
        #[structopt(parse(from_os_str))]
        install_path: Option<PathBuf>,
    },
    Update {
        #[structopt(short, long)]
        workers: Option<u8>,
    },
    Install {
        #[structopt(short, long)]
        workers: Option<u8>,
        #[structopt(long, parse(try_from_str = parse_version), conflicts_with("interactive"))]
        version: Option<SemVer>,
        #[structopt(short, long)]
        interactive: bool,
    },
    Download {
        #[structopt(parse(from_os_str))]
        output: PathBuf,
        #[structopt(short, long, default_value = "2")]
        workers: u8,
        #[structopt(long, parse(try_from_str = parse_version), conflicts_with("interactive"))]
        version: Option<SemVer>,
        #[structopt(short, long)]
        interactive: bool,
    },
}

impl Opt {
    pub fn run(&self) -> Result<GoVersion> {
        let term = Term::stdout();
        let versions = GoVersions::new(None)?;
        let golang = {
            if let Some(vers) = &self.version {
                let chosen: GoVersion = versions.chosen_version(vers.clone())?;
                chosen
            } else if self.interactive {
                let vers = ask_for_version(&term, &versions)?;
                let chosen: GoVersion = versions.chosen_version(vers)?;
                chosen
            } else if self.update {
                let if_latest = versions.check_local_latest(None)?;
                if !if_latest {
                    versions.latest()
                } else {
                    paris::success!("You have the latest version installed");
                    quit::with_code(0);
                }
            } else {
                versions.latest()
            }
        };
        paris::info!(
            "<b><bright blue>Filename: go{}.{}</></b>",
            golang.version,
            FILE_EXT.as_str()
        );
        Ok(golang)
    }
}

fn parse_version(src: &str) -> Result<SemVer> {
    SemVer::new(src).ok_or(Error::VersParse)
}

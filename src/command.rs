use crate::ask_for_version;
use crate::consts::FILE_EXT;
use crate::error::Error;
use crate::goversion::{check_git, GoVersion, GoVersions};
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
}

impl Opt {
    pub fn run(&self) -> Result<GoVersion> {
        let term = Term::stdout();
        let git_present = check_git();
        let versions = GoVersions::new(None)?;
        let golang = {
            if let Some(vers) = &self.version {
                let chosen: GoVersion = versions.chosen_version(vers.clone())?;
                chosen
            } else if self.interactive && git_present {
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
            FILE_EXT
        );
        Ok(golang)
    }
}

fn parse_version(src: &str) -> Result<SemVer> {
    SemVer::new(src).ok_or(Error::VersParse)
}

use crate::ask_for_version;
use crate::error::Error;
use crate::goversion::{check_git, GoVersion, GoVersions};
use crate::Result;
use dialoguer::console::Term;
use std::path::PathBuf;
use structopt::StructOpt;
use versions::Versioning;

#[derive(Debug, StructOpt)]
pub(crate) struct Opt {
    #[structopt(parse(from_os_str))]
    pub output: PathBuf,
    #[structopt(short, long, default_value = "2")]
    pub workers: u8,
    #[structopt(short, long, conflicts_with("version"), conflicts_with("interactive"))]
    pub update: bool,
    #[structopt(long, parse(try_from_str = parse_version), conflicts_with("interactive"))]
    pub version: Option<Versioning>,
    #[structopt(short, long)]
    pub interactive: bool,
    #[structopt(short, long)]
    pub git: bool,
}

impl Opt {
    pub async fn run(&self) -> Result<GoVersion> {
        let term = Term::stdout();
        let git_present = check_git();
        println!("ARCH: {}", std::env::consts::ARCH);
        println!("File ext: {}", crate::consts::FILE_EXT);
        let versions: GoVersions = if git_present {
            GoVersions::new(self.git, None).await?
        } else {
            GoVersions::new(false, None).await?
        };
        let golang = {
            if let Some(vers) = &self.version {
                let chosen: GoVersion = versions.chosen_version(vers.clone())?;
                chosen
            } else if self.interactive && git_present {
                let vers = ask_for_version(&term, &versions).await?;
                let chosen: GoVersion = versions.chosen_version(vers)?;
                chosen
            } else if self.update {
                let if_latest = versions.check_local_latest(None).await?;
                if !if_latest {
                    versions.latest()
                } else {
                    leg::success("You have the latest version installed", None, None).await;
                    quit::with_code(0);
                }
            } else {
                versions.latest()
            }
        };
        Ok(golang)
    }
}

fn parse_version(src: &str) -> Result<Versioning> {
    Versioning::new(src).ok_or(Error::VersParse)
}

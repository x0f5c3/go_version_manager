use clap::Parser;
use clap::Subcommand;

use {completions::Completions, download::Download, init::Init, install::Install, update::Update};

// use crate::Result;
use anyhow::Result;
use shadow_rs::shadow;

shadow!(shadow);
mod completions;
mod download;
mod init;
mod install;
mod update;
mod utils;


#[derive(Debug, Parser)]
#[clap(name = "go_version_manager")]
#[clap(author = "x0f5c3 <x0f5c3@tutanota.com>")]
#[clap(version = shadow::CLAP_LONG_VERSION)]
/// I will download Go language install and check its hash to verify I did it correctly
///
/// Keep calm and carry on
pub(crate) struct Opt {
    #[clap(subcommand)]
    pub(crate) subcommand: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Init(Init),
    Update(Update),
    Install(Install),
    Download(Download),
    Completions(Completions),
}

impl Command {
    pub fn run(self) -> Result<()> {
        match self {
            Self::Download(d) => d.run(),
            Self::Init(i) => i.run(),
            Self::Update(u) => u.run(),
            Self::Completions(c) => c.run(),
            Self::Install(i) => i.run(),
        }
    }
}

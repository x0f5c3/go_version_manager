use structopt::StructOpt;
use clap::Parser;

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
/// I will download Go language install and check its hash to verify I did it correctly{n}Keep calm and carry on
pub(crate) enum Command {
    #[clap(subcommand)]
    Init(Init),
    #[clap(subcommand)]
    Update(Update),
    #[clap(subcommand)]
    Install(Install),
    #[clap(subcommand)]
    Download(Download),
    #[clap(subcommand)]
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

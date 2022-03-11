use structopt::StructOpt;

use {completions::Completions, download::Download, init::Init, install::Install, update::Update};

// use crate::Result;
use anyhow::Result;

mod completions;
mod download;
mod init;
mod install;
mod update;
mod utils;

#[derive(Debug, StructOpt)]
/// I will download Go language install and check its hash to verify I did it correctly{n}Keep calm and carry on
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

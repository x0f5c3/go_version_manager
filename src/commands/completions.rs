use std::path::PathBuf;

use structopt::StructOpt;
use clap::Parser;
use anyhow::Result;
use clap_complete::generate_to;
use clap_complete::shells;

/// Generate completions
#[derive(Debug, Clone, Parser)]
pub(crate) struct Completions {
    shell: String,
    #[clap(parse(from_os_str))]
    out_dir: PathBuf,
}

impl Completions {
    pub(crate) fn run(self) -> Result<()> {
        generate_to(self.shell, &mut self.into(), "go_version_manager", self.out_dir);
        Ok(())
    }
}

use std::fs::File;
use std::path::PathBuf;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};

/// Generate completions
#[derive(Debug, Clone, Parser)]
pub(crate) struct Completions {
    #[clap(parse(try_from_str))]
    shell: Shell,
    #[clap(parse(from_os_str))]
    out_dir: PathBuf,
}

impl Completions {
    pub(crate) fn run(self) -> Result<()> {
        let mut out = File::create(self.out_dir.join("_go_version_manager"))?;
        generate(
            self.shell,
            &mut Self::command(),
            "go_version_manager",
            &mut out,
        );
        Ok(())
    }
}

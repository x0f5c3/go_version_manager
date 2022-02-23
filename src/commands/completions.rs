use std::path::PathBuf;

use structopt::StructOpt;

use anyhow::Result;

/// Generate completions
#[derive(Debug, Clone, StructOpt)]
pub(crate) struct Completions {
    shell: structopt::clap::Shell,
    #[structopt(parse(from_os_str))]
    out_dir: PathBuf,
}

impl Completions {
    pub(crate) fn run(self) -> Result<()> {
        let mut app: structopt::clap::App = Self::clap();
        app.gen_completions("go_version_manager", self.shell, self.out_dir);
        Ok(())
    }
}

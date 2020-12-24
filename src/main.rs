//! `go_version_manager` is a small program intended to download the latest or chosen golang version
//! from the official site also checking the checksum for the file
use colored::Colorize;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};
use error::Error;
use goversion::GoVersion;
use human_panic::setup_panic;
use std::path::PathBuf;
use structopt::StructOpt;
use versions::Versioning;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    output: PathBuf,
    #[structopt(short, long, default_value = "2")]
    workers: u8,
    #[structopt(long, parse(try_from_str = parse_version), conflicts_with("interactive"))]
    version: Option<Versioning>,
    #[structopt(short, long)]
    interactive: bool,
}

/// Reads output path from command line arguments
/// and downloads latest golang version to it
#[tokio::main]
#[quit::main]
async fn main() -> Result<(), Error> {
    setup_panic!();
    let opt = Opt::from_args();
    let term = Term::stdout();
    let golang = {
        if let Some(vers) = opt.version {
            goversion::GoVersion::version(vers).await?
        } else if opt.interactive {
            let vers = ask_for_version(&term)?;
            goversion::GoVersion::version(vers).await?
        } else {
            goversion::GoVersion::latest().await?
        }
    };
    format!("Downloading golang version {}", &golang.version);
    leg::info(
        &format!(
            "Downloading golang {}",
            &golang.version.to_string().green().bold()
        ),
        None,
        None,
    );
    let file_path = golang.download(opt.output, opt.workers).await?;
    let path_str = file_path.to_str().expect("Couldn't convert path to str").to_string();
    leg::success(
        &format!("Golang has been downloaded to {}", path_str),
        None,
        None,
    );

    Ok(())
}

fn parse_version(src: &str) -> Result<Versioning, Error> {
    Versioning::new(src).ok_or_else(|| Error::VersParse)
}

fn ask_for_version(term: &Term) -> Result<Versioning, Error> {
    let versions = GoVersion::get_versions()?[0..=40].to_vec();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&versions)
        .default(0)
        .paged(true)
        .interact_on_opt(term)?;
    if let Some(index) = selection {
        Ok(versions[index].clone())
    } else {
        leg::error(&format!("{}", "You didn't select anything".red().bold()), None, None);
        quit::with_code(127);
    }
}

mod error;
mod goversion;

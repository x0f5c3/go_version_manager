//! `go_version_manager` is a small program intended to download the latest or chosen golang version
//! from the official site also checking the checksum for the file
use console::{Style, Term};
use human_panic::setup_panic;
use std::path::PathBuf;
use structopt::StructOpt;
use error::Error;


#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    output: PathBuf,
    #[structopt(short,long, default_value = "2")]
    workers: u8,
}

/// Reads output path from command line arguments
/// and downloads latest golang version to it
#[tokio::main]
#[quit::main]
async fn main() -> Result<(), Error> {
    setup_panic!();
    let opt = Opt::from_args();
    let style = Style::new().green().bold();
    let golang = goversion::GoVersion::latest().await?;
    let term = Term::stdout();
    term.set_title(format!(
        "Downloading golang version {}",
        golang.version.clone()
    ));
    println!(
        "Downloading golang {}",
        style.apply_to(golang.version.clone())
    );
    let file_path = golang.download(opt.output, opt.workers).await?;
    let path_str = file_path.to_str().expect("Couldn't convert path to str");
    println!("Golang has been downloaded to {}", path_str);

    Ok(())
}
/// Golang version represented as a struct

mod goversion;
mod error;
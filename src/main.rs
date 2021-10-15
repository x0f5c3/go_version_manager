#![allow(dead_code, clippy::enum_variant_names)]
//! `go_version_manager` is a small program intended to download the latest or chosen golang version
//! from the official site also checking the checksum for the file
use colored::Colorize;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};
use error::Error;
use human_panic::setup_panic;
use versions::Versioning;
use crate::command::Opt;
use crate::error::Result;
use crate::goversion::GoVersions;
use crate::goversion::Downloaded;

/// Reads output path from command line arguments
/// and downloads latest golang version to it
#[tokio::main]
#[quit::main]
async fn main() -> Result<()> {
    setup_panic!();
    pretty_env_logger::init();
    let opt = Opt::new();
    let golang = opt.run().await?;
    format!("Downloading golang version {}", &golang.version);
    leg::info(
        &format!(
            "Downloading golang {}",
            &golang.version.to_string().green().bold()
        ),
        None,
        None,
    ).await;
    println!("DL_URL: {}", golang.dl_url);
    let file_path = golang.download(Some(opt.output), opt.workers).await?;
    if let Downloaded::File(path) = file_path {
        let path_str = path.to_str().ok_or(Error::PathBufErr)?;
        leg::success(
            &format!("Golang has been downloaded to {}", path_str),
            None,
            None,
        ).await;
    }
    Ok(())
}



async fn ask_for_version(term: &Term, versions: &GoVersions) -> Result<Versioning> {
    let versions = versions.versions.iter().map(|x| x.version.clone()).collect::<Vec<Versioning>>();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&versions)
        .default(0)
        .paged(true)
        .interact_on_opt(term)?;
    if let Some(index) = selection {
        Ok(versions[index].clone())
    } else {
        leg::error(
            &format!("{}", "You didn't select anything".red().bold()),
            None,
            None,
        ).await;
        quit::with_code(127);
    }
}

mod consts;
mod error;
mod github;
mod goversion;
mod command;
mod config;
mod decompressor;

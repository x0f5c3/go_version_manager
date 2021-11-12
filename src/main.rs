#![allow(dead_code, clippy::enum_variant_names)]
//! `go_version_manager` is a small program intended to download the latest or chosen golang version
//! from the official site also checking the checksum for the file
#[macro_use]
extern crate lazy_static;

use crate::command::Opt;
use crate::consts::FILE_EXT;
use crate::error::Result;
use crate::goversion::Downloaded;
use crate::goversion::GoVersions;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};
use error::Error;
use human_panic::setup_panic;
use versions::SemVer;

/// Reads output path from command line arguments
/// and downloads latest golang version to it
#[paw::main]
#[quit::main]
fn main(opt: Opt) -> Result<()> {
    let now = std::time::Instant::now();
    setup_panic!();
    lazy_static::initialize(&FILE_EXT);
    pretty_env_logger::init();
    let golang = opt.run()?;
    paris::info!(
        "<b><blue>Downloading golang version {}</></b>",
        &golang.version
    );
    let file_path = golang.download(Some(opt.output), opt.workers)?;
    if let Downloaded::File(path) = file_path {
        let path_str = path.to_str().ok_or(Error::PathBufErr)?;
        paris::success!(
            "<b><bright green>Golang has been downloaded to {}</></b>",
            path_str
        );
    }
    paris::info!("Execution time: {}s", now.elapsed().as_secs_f64());
    Ok(())
}

fn ask_for_version(term: &Term, versions: &GoVersions) -> Result<SemVer> {
    let versions = versions
        .versions
        .iter()
        .map(|x| x.version.clone())
        .collect::<Vec<SemVer>>();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&versions)
        .default(0)
        .paged(true)
        .interact_on_opt(term)?;
    if let Some(index) = selection {
        Ok(versions[index].clone())
    } else {
        paris::error!("<bold><red>You didn't select anything</red></bold>");
        quit::with_code(127);
    }
}

mod command;
mod config;
mod consts;
mod decompressor;
mod error;
mod goversion;

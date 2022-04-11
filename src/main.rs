#![allow(dead_code)]
//! `go_version_manager` is a small program intended to download the latest or chosen golang version
//! from the official site also checking the checksum for the file
#[macro_use]
extern crate lazy_static;

use console::Term;
use error::Error;
use human_panic::setup_panic;
pub(crate) use utils::{ask_for_version, init_consts};

use crate::commands::Opt;
use crate::consts::FILE_EXT;
use crate::goversion::GoVersions;
use crate::goversion::{Downloaded, GoVersion};
use crate::utils::check_and_ask;

use anyhow::Result;

/// Reads output path from command line arguments
/// and downloads latest golang version to it
#[quit::main]
fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    let now = std::time::Instant::now();
    setup_panic!();
    init_consts();
    pretty_env_logger::init();
    check_and_ask(&Term::stdout())?;
    // let res = opt.run();
    // if let Err(e) = res {
    //     paris::error!("Error: {}", e);
    // }
    let raw = GoVersions::raw_git_versions()?;
    let parsed = GoVersions::new(None)?;
    let latest = GoVersions::download_latest()?;
    paris::log!("Raw versions: {:?}", raw);
    paris::log!("Parsed versions: {:?}", parsed);
    paris::log!("Latest: {:?}", latest);
    #[cfg(debug_assertions)]
    paris::info!("Execution time: {}s", now.elapsed().as_secs_f64());
    Ok(())
}

mod commands;
mod config;
mod consts;
mod decompressor;
mod error;
mod github;
mod goversion;
mod installed;
mod utils;

#![allow(dead_code, clippy::enum_variant_names)]
//! `go_version_manager` is a small program intended to download the latest or chosen golang version
//! from the official site also checking the checksum for the file
#[macro_use]
extern crate lazy_static;

use human_panic::setup_panic;

use error::Error;
pub(crate) use utils::{ask_for_version, init_consts};

use crate::commands::Command;
use crate::consts::FILE_EXT;
use crate::error::Result;
use crate::goversion::Downloaded;
use crate::goversion::GoVersions;

/// Reads output path from command line arguments
/// and downloads latest golang version to it
#[paw::main]
#[quit::main]
fn main(opt: Command) -> Result<()> {
    #[cfg(debug_assertions)]
    let now = std::time::Instant::now();
    setup_panic!();
    init_consts();
    pretty_env_logger::init();
    let res = opt.run();
    if let Err(e) = res {
        paris::error!("Error: {}", e);
    }
    #[cfg(debug_assertions)]
    paris::info!("Execution time: {}s", now.elapsed().as_secs_f64());
    Ok(())
}

mod commands;
mod config;
mod consts;
mod decompressor;
mod error;
mod goversion;
mod installed;
mod utils;

#![allow(dead_code)]
//! `go_version_manager` is a small program intended to download the latest or chosen golang version
//! from the official site also checking the checksum for the file
#[macro_use]
extern crate lazy_static;

use human_panic::setup_panic;
pub(crate) use utils::{ask_for_version, init_consts};

use crate::goversion::Downloaded;
use crate::goversion::GoVersions;

use anyhow::anyhow;
use anyhow::Result;

use clap::Parser;
use commands::Opt;

/// Reads output path from command line arguments
/// and downloads latest golang version to it
#[quit::main]
fn main() -> Result<()> {
    setup_panic!();
    let opt = Opt::try_parse()?;
    #[cfg(debug_assertions)]
    let now = std::time::Instant::now();
    init_consts();
    tracing_subscriber::fmt()
        .pretty()
        .try_init()
        .map_err(|x| anyhow!("Failed to init the tracing subscriber: {}", x.to_string()))?;
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
mod envs;
mod goversion;
mod utils;

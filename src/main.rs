#![allow(dead_code)]
//! `go_version_manager` is a small program intended to download the latest or chosen golang version
//! from the official site also checking the checksum for the file
#[macro_use]
extern crate lazy_static;

use console::Term;
use error::Error;
use human_panic::setup_panic;
use serde::Serialize;
pub(crate) use utils::{ask_for_version, init_consts};

use crate::consts::FILE_EXT;
use crate::goversion::Downloaded;
use crate::goversion::GoVersions;
use crate::utils::check_and_ask;
use chrono::{DateTime, Utc};
use std::time::SystemTime;

use crate::error::Result;

use anyhow::Context;

use semver::Version;
// use semver::Version;
use tokio_uring::fs::File;

/// Reads output path from command line arguments
/// and downloads latest golang version to it
#[quit::main]
fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    let now = std::time::Instant::now();
    setup_panic!();
    init_consts();
    tracing_subscriber::fmt().pretty().try_init().unwrap();
    // pretty_env_logger::init();
    check_and_ask(&Term::stdout())?;
    // let res = opt.run();
    // if let Err(e) = res {
    //     paris::error!("Error: {}", e);
    // }
    let raw = GoVersions::raw_git_versions()?;
    let now_date = DateTime::<Utc>::from(SystemTime::now());
    let stamp = now_date.format("%H-%M-%S").to_string();
    let mut parsed = GoVersions::parsed_versions()?;
    parsed.sort_unstable_by(|a, b| b.cmp(a));
    let latest = parsed.first().context("Failed to get latest")?.clone();
    #[derive(Serialize)]
    struct ToSer {
        raw: Vec<String>,
        parsed: Vec<Version>,
        latest: Version,
    }
    let to_ser = ToSer {
        raw,
        parsed,
        latest,
    };
    let filename = format!("./test_{}.toml", stamp);
    let sered = toml::to_string_pretty(&to_ser).context("Couldn't serialize")?;
    tokio_uring::start(async {
        paris::info!("Writing {} to {}", sered, filename);
        let file = File::create(filename).await?;
        let buf = sered.into_bytes();
        let (res, _) = file.write_at(buf, 0).await;
        println!("Wrote {} bytes using io-uring", res?);
        std::result::Result::<(), anyhow::Error>::Ok(())
    })?;
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

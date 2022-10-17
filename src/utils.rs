use crate::consts::{CONFIG_DIR, CONFIG_PATH, CURRENT_INSTALL, DEFAULT_INSTALL, VERSION_LIST};
use crate::goversion::GoVersion;
use crate::GoVersions;
use anyhow::{Context, Result};
use console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use semver::Version;
use std::io::ErrorKind;

use crate::consts::PATH_SEPERATOR;
use rayon::prelude::{ParallelBridge, ParallelIterator};

use std::path::Path;

pub(crate) fn init_consts() {
    lazy_static::initialize(&CURRENT_INSTALL);
    lazy_static::initialize(&CONFIG_PATH);
    lazy_static::initialize(&DEFAULT_INSTALL);
    lazy_static::initialize(&VERSION_LIST);
    lazy_static::initialize(&CONFIG_DIR);
}

pub(crate) fn ask_for_version(term: &Term, versions: &GoVersions) -> Result<GoVersion> {
    let versions = versions.versions.to_vec();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&versions)
        .default(0)
        .interact_on_opt(term)?;
    if let Some(index) = selection {
        versions.get(index).cloned().context("No version available")
    } else {
        paris::error!("<bold><red>You didn't select anything</red></bold>");
        quit::with_code(127);
    }
}

pub(crate) fn get_local_version(path: &Path) -> Result<Option<Version>> {
    duct::cmd!(
        path.join("bin/go").to_str().context("No go exec")?,
        "version"
    )
    .read()
    .map(|x| {
        x.split(' ')
            .nth(2)
            .and_then(|x| Version::parse(&x.replace("go", "")).ok())
    })
    .or_else(|x| {
        if x.kind() == ErrorKind::NotFound {
            return Ok(None);
        }
        Err(x.into())
    })
}

pub(super) fn check_writable(p: &Path) -> Result<bool> {
    let res = std::fs::write(p.join("test"), "test");
    if let Err(e) = res {
        if e.kind() == ErrorKind::PermissionDenied {
            Ok(false)
        } else {
            Err(e.into())
        }
    } else {
        std::fs::remove_file(p.join("test"))?;
        Ok(true)
    }
}

pub(super) fn parse_version(src: &str) -> Result<Version> {
    Version::parse(src).context("Failed to parse version")
}

pub(super) fn check_in_path(p: &Path) -> Result<bool> {
    let user_path = std::env::var("PATH")?;
    let p_str = p.to_str().context("Failed to convert path to string")?;
    Ok(user_path
        .split(PATH_SEPERATOR)
        .par_bridge()
        .any(|x| x == p_str))
}

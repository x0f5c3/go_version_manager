use crate::consts::{CONFIG_DIR, CONFIG_PATH, CURRENT_INSTALL, DEFAULT_INSTALL, VERSION_LIST};
use crate::Error;
use crate::{GoVersions, FILE_EXT};
use anyhow::Result;
use console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use semver::Version;

use std::path::Path;

pub(crate) fn init_consts() {
    lazy_static::initialize(&FILE_EXT);
    lazy_static::initialize(&CURRENT_INSTALL);
    lazy_static::initialize(&CONFIG_PATH);
    lazy_static::initialize(&DEFAULT_INSTALL);
    lazy_static::initialize(&VERSION_LIST);
    lazy_static::initialize(&CONFIG_DIR);
}

pub(crate) fn ask_for_version(term: &Term, versions: &GoVersions) -> Result<Version> {
    let versions = versions
        .versions
        .iter()
        .map(|x| x.version.clone())
        .collect::<Vec<Version>>();
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

pub(crate) fn get_local_version(path: &Path) -> Result<Option<Version>> {
    duct::cmd!(
        path.join("bin/go").to_str().ok_or(Error::PathBufErr)?,
        "version"
    )
    .read()
    .map(|x| {
        x.split(' ')
            .nth(2)
            .and_then(|x| Version::parse(&x.replace("go", "")).ok())
    })
    .or_else(|x| {
        if x.kind() == std::io::ErrorKind::NotFound {
            return Ok(None);
        }
        Err(x.into())
    })
}

use crate::consts::{CONFIG_DIR, CONFIG_PATH, CURRENT_INSTALL, DEFAULT_INSTALL, VERSION_LIST};
use crate::Result;
use crate::{GoVersions, FILE_EXT};
use console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use std::path::PathBuf;
use versions::SemVer;

pub(crate) fn get_local_path() -> Option<PathBuf> {
    let cmd = if cfg!(windows) {
        "Get-Command"
    } else {
        "which"
    };
    duct::cmd!(cmd, "go")
        .read()
        .into_iter()
        .filter_map(|x| {
            PathBuf::from(x)
                .parent()
                .into_iter()
                .filter_map(|x| x.parent())
                .next()
                .map(|x| x.to_path_buf())
        })
        .next()
}

pub(crate) fn init_consts() {
    lazy_static::initialize(&FILE_EXT);
    lazy_static::initialize(&CURRENT_INSTALL);
    lazy_static::initialize(&CONFIG_PATH);
    lazy_static::initialize(&DEFAULT_INSTALL);
    lazy_static::initialize(&VERSION_LIST);
    lazy_static::initialize(&CONFIG_DIR);
}

pub(crate) fn ask_for_version(term: &Term, versions: &GoVersions) -> Result<SemVer> {
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

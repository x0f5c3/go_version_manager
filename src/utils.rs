use crate::consts::{CONFIG_DIR, CONFIG_PATH, CURRENT_INSTALL, DEFAULT_INSTALL, VERSION_LIST};
use crate::Error;
use crate::{GoVersions, FILE_EXT};
use anyhow::Result;
use console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use self_update::backends::github::Update;
use self_update::cargo_crate_version;
use self_update::update::ReleaseUpdate;
use semver::Version;
use std::fmt;
use std::fmt::Formatter;
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

pub(crate) enum ShouldUpdate {
    Yes(Box<dyn ReleaseUpdate>),
    No,
}

impl fmt::Display for ShouldUpdate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Yes(u) => write!(f, "Update available: {:?}", u.get_latest_release()),
            Self::No => write!(f, "You have the latest version"),
        }
    }
}

impl ShouldUpdate {
    fn new(update: Option<Box<dyn ReleaseUpdate>>) -> Self {
        if let Some(up) = update {
            Self::Yes(up)
        } else {
            Self::No
        }
    }
}

pub(crate) fn check_and_ask(term: &Term) -> Result<()> {
    let should = check_self_update()?;
    if let ShouldUpdate::Yes(up) = should {
        let answer = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("There's an update available\nYour version: {}\nLatest version: {}\nDo you want to update? [Y/n]", up.current_version(), up.get_latest_release()?.version))
            .default(true).interact_on_opt(term)?.ok_or(Error::NoVersion)?;
        if answer {
            let upres = up.update()?;
            if upres.updated() {
                paris::success!(
                    "<b><green>Successfully updated to version {}</></b>",
                    upres.version()
                );
                return Ok(());
            } else {
                paris::error!(
                    "<b><red>Failed to update, current version {}</></b>",
                    upres.version()
                )
            }
            return Ok(());
        } else {
            paris::info!("Ok, I will remind you next time")
        }
        return Ok(());
    }
    Ok(())
}

pub(crate) fn check_self_update() -> Result<ShouldUpdate> {
    let up = Update::configure()
        .repo_owner("x0f5c3")
        .repo_name("go_version_manager")
        .bin_name("go_version_manager")
        .current_version(cargo_crate_version!())
        .build()?;
    let rel = up.get_latest_release()?;
    if self_update::version::bump_is_greater(&up.current_version(), &rel.version)? {
        Ok(ShouldUpdate::new(Some(up)))
    } else {
        Ok(ShouldUpdate::No)
    }
}

pub(crate) fn self_update() -> Result<()> {
    let status = Update::configure()
        .repo_owner("x0f5c3")
        .repo_name("go_version_manager")
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;
    if status.updated() {
        paris::success!("Updated the binary to version {}", status.version());
    }
    Ok(())
}

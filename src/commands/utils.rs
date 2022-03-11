use std::io::ErrorKind;
use std::path::Path;

use itertools::Itertools;
use versions::SemVer;

use crate::consts::PATH_SEPERATOR;
use crate::error::Error;
// use crate::Result;
use anyhow::Result;

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

pub(super) fn parse_version(src: &str) -> Result<SemVer> {
    SemVer::new(src).ok_or_else(|| Error::VersParse.into())
}

pub(super) fn check_in_path(p: &Path) -> Result<bool> {
    let user_path = std::env::var("PATH")?;
    Ok(user_path
        .split(PATH_SEPERATOR)
        .contains(&p.to_str().ok_or(Error::PathBufErr)?))
}

use crate::consts::PATH_SEPERATOR;
use crate::goversion::GoVersion;
// use crate::Result;
use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use std::path::{Path, PathBuf};

pub(crate) struct Installed {
    path: PathBuf,
    version: GoVersion,
}

impl Installed {
    pub(crate) fn activate(&self) -> Result<()> {
        remove_set()?;
        add_to_path(&self.path)
    }
}

pub(crate) fn add_to_path(path: &Path) -> Result<()> {
    let path_var = std::env::var("PATH")?;
    std::env::set_var(
        "PATH",
        format!("{}:{}", path_var, path.join("bin").display()),
    );
    Ok(())
}

pub(crate) fn remove_from_path() -> Result<String> {
    let path_var = std::env::var("PATH")?;
    let filter = Regex::new(r#".*go[\\/]bin"#)?;
    Ok(path_var
        .split(PATH_SEPERATOR)
        .collect::<Vec<&str>>()
        .par_iter()
        .filter_map(|x| {
            if !filter.is_match(x) {
                return Some(*x);
            }
            None
        })
        .collect::<Vec<&str>>()
        .join(PATH_SEPERATOR))
}

pub(crate) fn remove_set() -> Result<()> {
    let cleaned = remove_from_path()?;
    std::env::set_var("PATH", cleaned);
    Ok(())
}

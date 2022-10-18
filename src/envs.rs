use crate::config::App;
use crate::consts::{env_setter, ENVS_DIR};
use anyhow::{Context, Result};
use rayon::prelude::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstalledEnv {
    pub version: Version,
    pub path: PathBuf,
}

impl InstalledEnv {
    pub fn new(path: &Path) -> Result<InstalledEnv> {
        let go_path = path.join("bin").join("go");
        let potential_env_file = path.join(".go.env");
        let version = if potential_env_file.exists() {
            Version::parse(
                &fs::read_to_string(potential_env_file).context("Can't read .go.env file")?,
            )
            .context("Can't parse version from .go.env file")?
        } else {
            Version::parse(
                &duct::cmd!(&go_path, "version")
                    .read()
                    .context("Can't get go version")?
                    .split(" ")
                    .nth(2)
                    .context("Can't get go version")?
                    .replace("go", ""),
            )
            .context("Can't parse version from go version")?
        };
        Ok(InstalledEnv {
            version,
            path: path.to_path_buf(),
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnvManager {
    app: App,
    env_dir: PathBuf,
    current: InstalledEnv,
    available: Vec<InstalledEnv>,
}

impl EnvManager {
    pub fn new() -> Result<Self> {
        let env_dir = env_dir()?;
        match from_list(&env_dir.as_path()) {
            Some(x) => Ok(x),
            None => {
                let current = InstalledEnv::new(&env_dir.join("current"))?;
                // let available = vec![current.clone()];
                let available = fs::read_dir(&env_dir)
                    .context("Can't read env dir")?
                    .par_bridge()
                    .filter_map(|x| {
                        if x.ok()?.file_name().to_str()?.contains("current") {
                            None
                        } else {
                            Some(InstalledEnv::new(x.ok()?.path().as_path()).ok()?)
                        }
                    })
                    .collect();
                let ret = EnvManager {
                    env_dir,
                    current,
                    available,
                };
                ret.save()?;
                Ok(ret)
            }
        }
    }
    pub fn save(&self) -> Result<()> {
        fs::write(
            self.env_dir.join("envs.toml"),
            toml::to_string_pretty(&self)?,
        )?;
        fs::write(
            self.env_dir.join(".go.env"),
            env_setter(self.current.path.join("bin").display()),
        )?;
        Ok(())
    }
}

fn env_dir() -> Result<PathBuf> {
    if !ENVS_DIR.exists() {
        fs::create_dir(ENVS_DIR.as_path()).context("Can't create env dir")?;
    }
    Ok(ENVS_DIR.to_path_buf())
}

fn check_for_list(path: &Path) -> bool {
    path.join("envs.toml").exists()
}

fn from_list(path: &Path) -> Option<EnvManager> {
    if !check_for_list(path) {
        None
    } else {
        let list = fs::read_to_string(path.join("envs.toml")).ok()?;
        let mut ret: EnvManager = toml::from_str(&list).ok()?;
        if ret.env_dir != path {
            ret.env_dir = path.to_path_buf();
        }
        Some(ret)
    }
}

// fn switch_env(path: &Path) -> Result<()> {
//     let bin_path = path.push("bin");
//     let path = path.to_str().unwrap();
//     std::env::set_var("PATH", path);
//     Ok(())
// }

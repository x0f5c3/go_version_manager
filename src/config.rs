use std::path::PathBuf;
use crate::goversion::GoVersion;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Config {
    install_path: PathBuf,
    current: GoVersion,
}
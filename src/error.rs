use std::env::VarError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IOError")]
    IOErr(#[from] std::io::Error),
    #[error("Manic error: {0}")]
    ManicErr(#[from] manic::ManicError),
    #[error("Failed to convert Pathbuf to str")]
    PathBufErr,
    #[error("Failed to get version")]
    NoVersion,
    #[error("No sha256 found")]
    NoSha,
    #[error("Failed to parse version")]
    VersParse,
    #[error("JSON error: {0}")]
    JSONErr(#[from] serde_json::Error),
    #[error("Git error: {0}")]
    GITErr(#[from] git2::Error),
    #[cfg(target_os = "windows")]
    #[error("Zip error: {0}")]
    ZIPErr(#[from] zip::result::ZipError),
    #[error("ENV error: {0}")]
    VARErr(#[from] VarError),
    #[error("Permission denied, restart the program as root")]
    NOPerm,
    #[error("Update failed {0}")]
    UpdateErr(#[from] self_update::errors::Error),
    #[error("Regex error: {0}")]
    RegexErr(#[from] regex::Error),
}

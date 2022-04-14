use anyhow::anyhow;
use std::env::VarError;
use std::fmt;
use std::fmt::Formatter;
use std::path::PathBuf;
use thiserror::Error;
use tracing_error::SpanTrace;

pub type Result<T> = std::result::Result<T, WrappedError>;

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
    #[error("TOML serializer error: {0}")]
    TOMLErr(#[from] toml::ser::Error),
    #[error("Can't find project dirs")]
    NOProjectDir,
}

#[derive(Debug)]
pub struct WrappedError {
    context: SpanTrace,
    err: anyhow::Error,
}

impl fmt::Display for WrappedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.err.fmt(f)?;
        self.context.fmt(f)
    }
}

impl std::error::Error for WrappedError {}

impl WrappedError {
    pub fn new(msg: String) -> Self {
        Self {
            context: SpanTrace::capture(),
            err: anyhow!(msg),
        }
    }
}

impl From<anyhow::Error> for WrappedError {
    fn from(e: anyhow::Error) -> Self {
        Self {
            context: SpanTrace::capture(),
            err: e,
        }
    }
}

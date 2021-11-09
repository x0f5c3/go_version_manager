use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IOError")]
    IOError(#[from] std::io::Error),
    #[error("Reqwest error: {0}")]
    ReqError(#[from] reqwest::Error),
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
    #[error("Failed to deserialize from TOML: {0}")]
    TOMLDeErr(#[from] toml_edit::de::Error),
    #[error("Failed to serialize to TOML: {0}")]
    TOMLSeErr(#[from] toml_edit::ser::Error),
    #[error("JSON error: {0}")]
    JSONErr(#[from] serde_json::Error),
    #[error("Git error: {0}")]
    GITErr(#[from] git2::Error),
    #[cfg(target_os = "windows")]
    #[error("Zip error: {0}")]
    ZIPErr(#[from] zip::result::ZipError),
}

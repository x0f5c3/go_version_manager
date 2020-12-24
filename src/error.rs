use thiserror::Error;




#[derive(Debug, Error)]
pub enum Error {
    #[error("IOError")]
    IOError(#[from] std::io::Error),
    #[error("Reqwest error: {0}")]
    ReqError(#[from] reqwest::Error),
    #[error("Manic error: {0}")]
    ManicErr(#[from] manic::Error),
    #[error("Failed to convert Pathbuf to str")]
    PathBufErr,
    #[error("Failed to get version")]
    NoVersion,
    #[error("No sha256 found")]
    NoSha,
    #[error("Failed to parse version")]
    VersParse,

}
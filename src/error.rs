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
    #[error("Git error: {0}")]
    GitError(#[from] git2::Error),
    #[error("No sha256 found")]
    NoSha,

}
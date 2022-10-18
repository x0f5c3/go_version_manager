// use crate::{Error, Result};
use anyhow::Context;
use anyhow::Result;
use std::io::{BufRead, Read, Seek};
use std::path::Path;
use tracing::instrument;

#[cfg(unix)]
use flate2::bufread::GzDecoder;
#[cfg(unix)]
use tar::Archive;

pub struct ToDecompress<R>
where
    R: Read + Seek + BufRead,
{
    #[cfg(windows)]
    decompressor: zip::ZipArchive<R>,
    #[cfg(unix)]
    decompressor: Archive<GzDecoder<R>>,
}

impl<R: Read + Seek + BufRead> ToDecompress<R> {
    #[cfg(target_os = "windows")]
    pub(crate) fn new(inner: R) -> Self {
        Self {
            decompressor: zip::ZipArchive::new(inner)?,
        }
    }
    #[cfg(not(target_os = "windows"))]
    pub(crate) fn new(inner: R) -> Self {
        let dec = Archive::new(GzDecoder::new(inner));
        Self { decompressor: dec }
    }
    #[cfg(windows)]
    #[instrument(skip(self))]
    pub(crate) fn extract(&mut self, path: &Path) -> Result<()> {
        self.decompressor
            .extract(path.parent().context("No parent")?)
            .context("Unpacking error")
    }
    #[cfg(unix)]
    #[instrument(skip(self))]
    pub(crate) fn extract(&mut self, path: &Path) -> Result<()> {
        self.decompressor
            .unpack(path.parent().context("No parent")?)
            .context("Unpacking error")
    }
}

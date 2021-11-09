use crate::{Error, Result};
use std::io::{BufRead, Read, Seek};
use std::path::Path;

#[cfg(not(target_os = "windows"))]
use flate2::bufread::GzDecoder;
#[cfg(not(target_os = "windows"))]
use tar::Archive;

pub struct ToDecompress<R>
where
    R: Read + Seek + BufRead,
{
    #[cfg(target_os = "windows")]
    decompressor: zip::ZipArchive<R>,
    #[cfg(not(target_os = "windows"))]
    decompressor: Archive<GzDecoder<R>>,
}

impl<R: Read + Seek + BufRead> ToDecompress<R> {
    #[cfg(target_os = "windows")]
    pub(crate) fn new(inner: R) -> Result<Self> {
        Ok(Self {
            decompressor: zip::ZipArchive::new(inner)?,
        })
    }
    #[cfg(not(target_os = "windows"))]
    pub(crate) fn new(inner: R) -> Result<Self> {
        let dec = tar::Archive::new(GzDecoder::new(inner));
        Ok(Self { decompressor: dec })
    }
    #[cfg(target_os = "windows")]
    pub(crate) fn extract(&mut self, path: &Path) -> Result<()> {
        self.decompressor.extract(path).map_err(Error::ZIPErr)
    }
    #[cfg(not(target_os = "windows"))]
    pub(crate) fn extract(&mut self, path: &Path) -> Result<()> {
        self.decompressor.unpack(path).map_err(Error::IOError)
    }
}

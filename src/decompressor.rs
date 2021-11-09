use std::io::{Read, Seek};

#[cfg(not(target_os = "windows"))]
use async_compression::tokio::bufread::GzipDecoder;
use tokio::io::AsyncBufRead;

struct ToDecompress<R>
where
    R: Read + Seek + AsyncBufRead,
{
    #[cfg(target_os = "windows")]
    decompressor: zip::ZipArchive<R>,
    #[cfg(not(target_os = "windows"))]
    decompressor: GzipDecoder<R>,
}

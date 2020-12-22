use reqwest::Client;
use std::path::PathBuf;
use indicatif::{ProgressBar,ProgressStyle};
use log::debug;
use futures::future::join_all;
use reqwest::header::{CONTENT_LENGTH, RANGE};
use thiserror::Error;
use std::num::ParseIntError;
use tokio::{fs::File, io};
use tokio::prelude::*;


pub async fn get_length(url: &str, client: &Client) -> Result<u64, Error> {
    let head_req = client.head(url).send().await?;
    let headers = head_req.headers();
    debug!("Received head response: {:?}", headers);
    headers[CONTENT_LENGTH].to_str()?.parse::<u64>().map_err(Into::into)
}



pub fn get_filename(url: &str) -> Result<String, Error> {
    let parsed_url = reqwest::Url::parse(url)?;
    parsed_url.path_segments().and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name.to_string()) })
        .ok_or(Error::NoFilename(url.to_string()))
}

async fn download_chunk(client: &Client, url: &str, val: String, path: PathBuf, pb: ProgressBar) -> Result<(), Error> {

    let mut resp = client.get(url).header(RANGE, val).send().await?;
    let mut file = File::create(path).await?;
    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).await?;
        pb.inc(chunk.len() as u64);
    }
    file.sync_all().await?;
    file.flush().await?;
    Ok(())
}



pub async fn download(client: &Client, url: &str, path: String, workers: usize, temp: bool) -> Result<(), Error> {
    let length = get_length(url, client).await?;
    let pb = {
        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-");
        let pb = ProgressBar::new(length);
        pb.set_style(style);
        pb
    };

    let dl_dir = if temp {
        let tmp_dir = tempfile::tempdir()?;
        tmp_dir.path().to_owned()
    } else {
        PathBuf::from("./")
    };
    let mb = length / 1000000;
    let arr: Vec<u64> = (0..=mb).collect();
    let mut result = {
        let name = get_filename(url)?;
        let file_path = std::path::Path::new(&path).join(name);
        File::create(file_path).await?
    };
    let mut hndl_vec = Vec::new();
    let mut cnt: u32 = 1;
    for chunk in arr.chunks(mb as usize / workers) {
        let file_name = format!("part{}", cnt);
        let file_path = dl_dir.join(file_name);
        let low = chunk.first().unwrap() * 1000000;
        let hi = chunk.last().unwrap() * 1000000;
        let val = if low == hi {
            format!("bytes={}-", hi)
        } else {
            format!("bytes={}-{}", low, hi)
        };
        hndl_vec.push(download_chunk(client, url, val, file_path, pb.clone()));
            cnt += 1;
            }
    join_all(hndl_vec).await;
    pb.finish();
    for i in 1..cnt {
        let part_name = format!("./part{}", i);
        let part_path = dl_dir.join(&part_name);
        let mut buf = Vec::new();
        let mut part = File::open(&part_path).await?;
        part.read_to_end(&mut buf).await?;
        result.write(&buf).await?;
        result.sync_all().await?;
        result.flush().await?;
        tokio::fs::remove_file(&part_path).await?;
    }
    result.sync_all().await?;
    result.flush().await?;

    Ok(())
}









#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to parse content-length")]
    LenParse(#[from] ParseIntError),
    #[error("Tokio IO error: {0}")]
    TokioIOError(#[from] io::Error),
    #[error("Reqwest error: {0}")]
    NetError(#[from] reqwest::Error),
    #[error(transparent)]
    ToStr(#[from] reqwest::header::ToStrError),
    #[error("No filename in url")]
    NoFilename(String),
    #[error("Failed to parse URL")]
    UrlParseError(#[from] url::ParseError),


}
    

use indicatif::ProgressBar;
use reqwest::Client;
use reqwest::header::{CONTENT_LENGTH, RANGE};
use sha2::{Digest, Sha256};
use std::error::Error;
use tokio::fs::File;
use tokio::prelude::*;
use log::info;
use futures::future::join_all;
const TEST_HASH: &'static str = "9ba07a4b089767fe3bf553a2788b97ea1909a724f67d8410b18048e845eec3e8";


#[tokio::main(core_threads = 6)]
async fn main() -> Result<(), Box<dyn Error>> {
    human_panic::setup_panic!();
    pretty_env_logger::init();
    let style = indicatif::ProgressStyle::default_bar()
        .template("{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .progress_chars("#>-");
    let url = "http://127.0.0.1:8080/kraken.deb";
    let tmp_dir = tempfile::tempdir()?;
    let client = Client::builder().gzip(true).build()?;
    let head_req = client.head(url).send().await?;
    info!("Made head request: {:?}", head_req);
    let headers = head_req.headers();
    let mut hash = Sha256::new();
    let length: u64 = headers
        .get(CONTENT_LENGTH)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
        .parse()
        .unwrap();
    info!("File size is {}B", length);
    let arr: Vec<u64> = (0..=length).collect();
    let mut result = {
        let res_name = head_req
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("test.deb");
        info!("Output filename: {}", res_name);
        File::create(format!("./{}",res_name)).await?
    };
    info!("Created output file {:?}", result);
    let mut hndl_vec = Vec::new();
    let pb = ProgressBar::new(length);
    pb.set_style(style);
    let mut cnt: u32 = 1;
    for chunk in arr.chunks((length / 5) as usize) {
        let file_name = std::path::PathBuf::from(format!("./part{}", cnt));
        let file_path = tmp_dir.path().join(&file_name);
        let low = chunk.first().unwrap();
        let hi = chunk.last().unwrap();
        let val = if low == hi {
            format!("bytes={}-", hi)
        } else {
            format!("bytes={}-{}", low, hi)
        };
        let cl1 = client.clone();
        hndl_vec.push(download_chunk(cl1, url, val, file_path,pb.clone()));
        info!("Started task {}", cnt);
        cnt += 1;

    }
    join_all(hndl_vec).await;
    pb.finish();
    for i in 1..cnt {
        let part_name = format!("./part{}", i);
        let part_path = tmp_dir.path().join(&part_name);
        let mut buf = Vec::new();
        let mut part = File::open(&part_path).await?;
        part.read_to_end(&mut buf).await?;
        result.write(&buf).await?;
        result.sync_all().await?;
        result.flush().await?;
        hash.update(&buf);
        std::fs::remove_file(&part_path)?;
    }
    result.sync_all().await?;
    result.flush().await?;

    let finally = hash.finalize();
    let formatted = format!("{:x}", finally);
    if formatted == TEST_HASH {
        println!("Succeded with hash: {}", formatted);
    } else {
        println!("Failed with hash: {}\n Wanted: {}", formatted, TEST_HASH);
    }

    Ok(())
}

async fn download_chunk(
    client: Client,
    url: &str,
    val: String,
    path: std::path::PathBuf,
    pb: ProgressBar,
) -> Result<(), Box<dyn Error>> {
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

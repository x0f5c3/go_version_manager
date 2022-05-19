use anyhow::{Context, Result};
use manic::Client;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde_json::Value;

pub fn get_tags() -> Result<Vec<String>> {
    let client = Client::builder().user_agent("rust_api").build()?;
    let resp: Value = client
        .get("https://api.github.com/repos/golang/go/tags")
        .send()?
        .json()?;
    Ok(resp
        .as_array()
        .context("Not an array")?
        .par_iter()
        .filter_map(|x| {
            // println!("{}", x);
            x.as_object()
        })
        .filter_map(|x| x.get("name"))
        .filter_map(|x| {
            println!("{}", x);
            x.as_str()
        })
        .filter(|x| x.contains("go"))
        .map(|x| x.to_string())
        .collect::<Vec<String>>())
}

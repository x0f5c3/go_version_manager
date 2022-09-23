use anyhow::{Context, Result};
use itertools::Itertools;
use manic::Url;
use rayon::prelude::*;
use regex::Regex;

pub fn get_all_pages(url: &str) -> Result<Vec<serde_json::Value>> {
    let reg = Regex::new(r"<https.*")?;
    let client = manic::Client::builder().user_agent("GH Api").build()?;
    let first_page = client
        .get(url)
        .query(&[("per_page", "100"), ("page", "1")])
        .send()?;
    let last_n = first_page
        .headers()
        .get("Link")
        .with_context(|| format!("Failed to get Link header from {:?}", first_page.headers()))?
        .to_str()?
        .split(';')
        .par_bridge()
        .into_par_iter()
        .filter(|x| x.contains("https"))
        .filter_map(|x| Url::parse(&reg.find(x)?.as_str().replace('<', "").replace('>', "")).ok())
        .filter_map(|x| {
            x.query_pairs()
                .map(|(x, y)| (x.to_string(), y.to_string()))
                .collect_vec()
                .last()
                .cloned()
        })
        .collect::<Vec<(String, String)>>()
        .last()
        .context("Can't get last page")?
        .1
        .to_string()
        .parse::<u8>()?;
    let mut res = vec![first_page.json()?];
    for i in 2..=last_n {
        res.push(
            client
                .get(url)
                .query(&[("per_page", "100"), ("page", &i.to_string())])
                .send()?
                .json()?,
        );
    }
    Ok(res)
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Tag {
    pub name: String,
    zipball_url: String,
    tarball_url: String,
    commit: Commit,
    node_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Commit {
    sha: String,
    url: String,
}

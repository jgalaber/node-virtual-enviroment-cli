use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NodeRelease {
    pub version: String,
    pub date: String,
    pub files: Vec<String>,
}

#![allow(unused)]

use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Deserialize)]
struct Outer {
    #[serde(rename = "music")]
    inner: Vec<Config>,
}

#[derive(Deserialize)]
pub struct Config {
    pub name: String,
    pub source_path: String,
    pub lyrics_first: Option<String>,
    pub lyrics_second: Option<String>,
    pub avatar: String,
    pub background: String,
}

impl Config {
    pub fn new(config_file: impl AsRef<Path>) -> Vec<Self> {
        let content = fs::read_to_string(config_file.as_ref()).unwrap();
        let Outer { inner } = toml::from_str(&content).unwrap();
        inner
    }
}

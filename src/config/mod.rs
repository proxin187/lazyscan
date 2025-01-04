use std::collections::HashMap;
use std::fs;

use serde::Deserialize;


#[derive(Debug, Clone, Deserialize)]
pub struct General {
    pub threads: usize,
    pub timeout: usize,
    pub log: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TargetOptions {
    pub version: String,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    File { path: String },
    Shodan { query: String },
    Crawler { queue: String, seeds: Vec<String> },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub general: General,
    pub source: Source,
    pub target: HashMap<String, TargetOptions>,
}

impl Config {
    pub fn new(file: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file)?;

        toml::from_str(&content).map_err(|err| err.into())
    }
}

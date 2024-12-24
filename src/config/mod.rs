use std::collections::HashMap;
use std::fs;

use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct General {
    pub threads: usize,
}

#[derive(Debug, Deserialize)]
pub struct Target {
    pub version: String,
    pub execute: Option<String>,
    pub log: Option<String>,
    pub misconfig: bool,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: General,
    pub target: HashMap<String, Target>,
}

impl Config {
    pub fn new(file: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file)?;

        toml::from_str(&content).map_err(|err| err.into())
    }
}



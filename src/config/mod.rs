use std::fs;

use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct General {
    threads: usize,
}

#[derive(Debug, Deserialize)]
pub struct Target {
    package: String,
    execute: Option<String>,
    log: Option<String>,
    misconfig: bool,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    general: General,
    target: Option<Vec<Target>>,
}

impl Config {
    pub fn new(file: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file)?;

        toml::from_str(&content).map_err(|err| err.into())
    }
}



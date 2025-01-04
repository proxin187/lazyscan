use crate::config::Config;
use crate::scan::Scanner;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use serde::Deserialize;
use log::*;

use std::time::Duration;
use std::net::Ipv4Addr;
use std::env;


#[derive(Debug, Deserialize)]
pub struct Host {
    ip: u32,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Response {
    Success {
        #[serde(rename = "matches")]
        hosts: Vec<Host>,
    },
    Error {
        error: String,
    },
}

pub struct Shodan {
    client: Client,
    scanner: Scanner,
    key: String,
    should_close: bool,
}

impl Shodan {
    pub fn new(config: &Config) -> Result<Shodan, Box<dyn std::error::Error>> {
        Ok(Shodan {
            client: Client::new(),
            scanner: Scanner::new(config),
            key: env::var("API_KEY")?,
            should_close: false,
        })
    }

    fn search(&self, query: &str, page: usize) -> Result<Response, Box<dyn std::error::Error>> {
        let result = self.client
            .get(format!(
                "https://api.shodan.io/shodan/host/search?key={}&query={}&page={}&facets=country",
                self.key, query, page
            ))
            .send()?;

        Ok(result.json()?)
    }

    pub fn run(&mut self, multi: MultiProgress, query: &str, timeout: u64) -> Result<(), Box<dyn std::error::Error>> {
        let style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")?.progress_chars("##-");
        let mut count = 0;

        while !self.should_close {
            info!("searching {}", count);

            match self.search(query, count)? {
                Response::Success { hosts } => {
                    let pb = multi.add(ProgressBar::new(hosts.len() as u64));

                    pb.set_style(style.clone());

                    for host in hosts {
                        let url = format!("http://{}", Ipv4Addr::from_bits(host.ip));

                        match self.client.get(&url).timeout(Duration::from_secs(timeout)).send() {
                            Ok(response) => {
                                self.scanner.scan(&url, response.headers())?;
                            },
                            Err(err) => {
                                error!("reqwest: {}", err);
                            },
                        }

                        pb.inc(1);
                    }

                    multi.remove(&pb);
                },
                Response::Error { error } => {
                    error!("shodan: {}", error);

                    self.should_close = true;
                },
            }

            count += 1;
        }

        Ok(())
    }
}



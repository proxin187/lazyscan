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
pub struct Search {
    #[serde(rename = "matches")]
    hosts: Vec<Host>,

    total: usize,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Response {
    Success {
        #[serde(flatten)]
        search: Search,
    },
    Error {
        error: String,
    },
}

pub struct Shodan {
    client: Client,
    scanner: Scanner,
    key: String,
}

impl Shodan {
    pub fn new(modules: Vec<String>) -> Result<Shodan, Box<dyn std::error::Error>> {
        Ok(Shodan {
            client: Client::new(),
            scanner: Scanner::new(modules),
            key: env::var("API_KEY")?,
        })
    }

    pub fn search(&self, query: &str, page: usize) -> Option<Search> {
        info!("searching query={}, page={}", query, page);

        let url = format!("https://api.shodan.io/shodan/host/search?key={}&query={}&page={}&facets=country", self.key, query, page);

        match self.client.get(&url).timeout(Duration::from_secs(60)).send().and_then(|response| response.json()) {
            Ok(response) => match response {
                Response::Success { search } => Some(search),
                Response::Error { error } => {
                    warn!("reached end of search: {}", error);

                    None
                },
            },
            Err(err) => {
                warn!("reqwest failed with `{}`, ignoring", err);

                None
            },
        }
    }

    pub fn pages(&mut self, query: &str) -> Vec<Search>  {
        match self.search(query, 0) {
            Some(search) => {
                info!("searching {} pages", search.total / 100);

                (0..search.total / 100)
                    .filter_map(|page| self.search(query, page))
                    .collect::<Vec<Search>>()
            },
            None => Vec::new(),
        }
    }

    pub fn run(&mut self, multi: MultiProgress, query: &str) -> Result<(), Box<dyn std::error::Error>> {
        let style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")?.progress_chars("##-");

        for search in self.pages(query) {
            let pb = multi.add(ProgressBar::new(search.hosts.len() as u64));

            pb.set_style(style.clone());

            for host in search.hosts {
                self.scanner.modules(format!("http://{}", Ipv4Addr::from_bits(host.ip)).as_str())?;

                pb.inc(1);
            }

            multi.remove(&pb);
        }

        Ok(())
    }
}



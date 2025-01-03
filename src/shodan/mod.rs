use indicatif::MultiProgress;
use reqwest::blocking::Client;

use std::env;

pub struct Shodan {
    client: Client,
    key: String,
}

impl Shodan {
    pub fn new() -> Result<Shodan, Box<dyn std::error::Error>> {
        Ok(Shodan {
            client: Client::new(),
            key: env::var("API_KEY")?,
        })
    }

    fn search(&self, query: &str, page: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let result = self.client
            .get(format!(
                "https://api.shodan.io/shodan/host/search?key={}&query={}&page={}&facets=country",
                self.key, query, page
            ))
            .send()?;

        // TODO: deserialize with json

        Ok(Vec::new())
    }

    pub fn run(&self, multi: MultiProgress, query: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

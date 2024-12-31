mod queue;

use crate::config::Config;
use crate::scan::Scanner;

use queue::{Queue, Drain, MemoryQueue};

use scraper::{Html, Selector};
use reqwest::blocking::Client;
use indicatif::{ProgressBar, ProgressStyle};

use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::sync::Arc;


pub struct Job {
    queue: Arc<dyn Queue + Send + Sync>,
    scanner: Arc<Scanner>,
    client: Client,
    pb: ProgressBar,
}

impl Job {
    pub fn new(queue: Arc<dyn Queue + Send + Sync>, scanner: Arc<Scanner>, pb: ProgressBar) -> Job {
        Job {
            queue,
            scanner,
            client: Client::new(),
            pb,
        }
    }

    fn encode(&self, base: &str, path: &str) -> String {
        path.starts_with("https://")
            .then(|| path.to_string())
            .unwrap_or(format!("{}/{}", base, path.trim_start_matches('/')))
    }

    pub fn perform(&self, drain: Arc<dyn Drain + Send + Sync>, timeout: usize) -> Result<(), Box<dyn std::error::Error>> {
        let selector = Selector::parse("a")?;

        while let Some(url) = drain.pop()? {
            let builder = self.client.get(&url)
                .timeout(Duration::from_secs(timeout as u64));

            match builder.send() {
                Ok(response) => {
                    self.scanner.scan(&url, response.headers());

                    let next = Html::parse_document(&response.text()?)
                        .select(&selector)
                        .filter_map(|element| element.attr("href").map(|value| self.encode(&url, value)))
                        .collect::<Vec<String>>();

                    self.queue.extend(next)?;
                },
                Err(_) => {},
            }

            self.pb.inc(1);
        }

        Ok(())
    }
}

pub struct Crawler {
    queue: Arc<dyn Queue + Send + Sync>,
    scanner: Arc<Scanner>,
    config: Config,
}

impl Crawler {
    pub fn new(config: Config) -> Crawler {
        Crawler {
            queue: Arc::new(MemoryQueue::new(config.general.seeds.clone())),
            scanner: Arc::new(Scanner::new(&config)),
            config,
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")?.progress_chars("##-");

        let mut layer: usize = 0;

        while let Ok(drain) = self.queue.drain() {
            let pb = ProgressBar::new(drain.len()? as u64);

            pb.set_style(style.clone());

            pb.set_message(format!("layer {}", layer));

            let mut handles: Vec<JoinHandle<()>> = Vec::new();

            for _ in 0..self.config.general.threads {
                let drain = drain.clone();
                let queue = self.queue.clone();
                let scanner = self.scanner.clone();
                let timeout = self.config.general.timeout;
                let pb = pb.clone();

                let handle = thread::spawn(move || {
                    let _ = Job::new(queue, scanner, pb).perform(drain, timeout);
                });

                handles.push(handle);
            }

            for handle in handles {
                let _ = handle.join();
            }

            pb.finish_with_message("layer done");

            layer += 1;
        }

        Ok(())
    }
}



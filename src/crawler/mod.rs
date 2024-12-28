use crate::config::Config;
use crate::scan::Scanner;

use scraper::{Html, Selector};
use reqwest::blocking::Client;
use indicatif::{ProgressBar, ProgressStyle};

use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;

macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().map_err(|_| Into::<Box<dyn std::error::Error>>::into("failed to lock"))
    }
}

#[derive(Clone)]
pub struct Queue {
    queue: Arc<Mutex<Vec<String>>>,
    domains: Arc<Mutex<HashMap<String, ()>>>,
}

impl Queue {
    pub fn new(seeds: Vec<String>) -> Queue {
        Queue {
            queue: Arc::new(Mutex::new(seeds)),
            domains: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn push(&self, new: String) -> Result<(), Box<dyn std::error::Error>> {
        let domain = new.split('/').take(3).collect::<String>();

        if lock!(self.domains)?.insert(domain, ()).is_none() {
            lock!(self.queue).map(|mut lock| lock.push(new))?;
        }

        Ok(())
    }

    pub fn extend(&self, extend: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        for new in extend {
            self.push(new)?;
        }

        Ok(())
    }

    pub fn drain(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut lock = lock!(self.queue)?;

        match lock.len() {
            0 => Err("empty queue".into()),
            _ => Ok(lock.drain(..).collect::<Vec<String>>()),
        }
    }
}

pub struct Job {
    queue: Queue,
    scanner: Arc<Scanner>,
    client: Client,
    pb: ProgressBar,
}

impl Job {
    pub fn new(queue: Queue, scanner: Arc<Scanner>, pb: ProgressBar) -> Job {
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

    pub fn perform(&self, urls: Vec<String>, timeout: usize) -> Result<(), Box<dyn std::error::Error>> {
        let selector = Selector::parse("a")?;

        for url in urls {
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
    queue: Queue,
    scanner: Arc<Scanner>,
    config: Config,
}

impl Crawler {
    pub fn new(config: Config) -> Crawler {
        Crawler {
            queue: Queue::new(config.general.seeds.clone()),
            scanner: Arc::new(Scanner::new(&config)),
            config,
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")?.progress_chars("##-");

        let mut layer: usize = 0;

        while let Ok(queue) = self.queue.drain() {
            let pb = ProgressBar::new(queue.len() as u64);

            pb.set_style(style.clone());

            pb.set_message(format!("layer {}", layer));

            let mut handles: Vec<JoinHandle<()>> = Vec::new();

            for chunk in queue.chunks(queue.len().div_ceil(self.config.general.threads)).map(|chunk| chunk.to_vec()) {
                let queue = self.queue.clone();
                let scanner = self.scanner.clone();
                let timeout = self.config.general.timeout;
                let pb = pb.clone();

                let handle = thread::spawn(move || {
                    let _ = Job::new(queue, scanner, pb).perform(chunk, timeout);
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



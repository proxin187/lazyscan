use crate::config::Config;

use scraper::{Html, Selector};
use reqwest::blocking;

use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex};

macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().map_err(|_| Into::<Box<dyn std::error::Error>>::into("failed to lock"))
    }
}

pub struct Job {
    queue: Arc<Mutex<Vec<String>>>,
}

impl Job {
    pub fn new(queue: Arc<Mutex<Vec<String>>>) -> Job {
        Job {
            queue,
        }
    }

    pub fn perform(&self, urls: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let selector = Selector::parse("a")?;

        let next = urls.iter()
            .filter_map(|url| {
                // TODO: this ignores if the user provides an invalid url

                blocking::get(url)
                    .and_then(|response| response.text())
                    .map(|text| {
                        Html::parse_document(&text)
                            .select(&selector)
                            .filter_map(|element| element.attr("href").map(|value| value.to_string()))
                            .collect::<Vec<String>>()
                    })
                    .ok()
            })
            .flatten()
            .collect::<Vec<String>>();

        lock!(self.queue)?.extend(next);

        Ok(())
    }
}

pub struct Crawler {
    queue: Arc<Mutex<Vec<String>>>,
    config: Config,
}

impl Crawler {
    pub fn new(seed: String, config: Config) -> Crawler {
        Crawler {
            queue: Arc::new(Mutex::new(vec![seed])),
            config,
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        while let Ok(queue) = lock!(self.queue).map(|mut lock| lock.drain(..).collect::<Vec<String>>()) {
            let mut handles: Vec<JoinHandle<()>> = Vec::new();

            println!("queue: {}", queue.len());

            for chunk in queue.chunks(queue.len().div_ceil(self.config.general.threads)).map(|chunk| chunk.to_vec()) {
                let queue = self.queue.clone();

                let handle = thread::spawn(move || {
                    let _ = Job::new(queue).perform(chunk);
                });

                handles.push(handle);
            }

            for handle in handles {
                let _ = handle.join();
            }
        }

        Ok(())
    }
}



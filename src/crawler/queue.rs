use std::collections::HashMap;
use std::sync::{Arc, Mutex};

macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().map_err(|_| Into::<Box<dyn std::error::Error>>::into("failed to lock"))
    }
}


pub trait Drain {
    fn next(&mut self) -> Option<String>;

    fn length(&self) -> usize;

    fn chunks(self, size: usize) -> Vec<Box<dyn Drain>>;
}

pub trait Queue {
    fn extend(&self, extend: Vec<String>) -> Result<(), Box<dyn std::error::Error>>;

    fn drain(&self) -> Result<Box<dyn Drain>, Box<dyn std::error::Error>>;
}


#[derive(Clone)]
pub struct MemoryQueue {
    queue: Arc<Mutex<Vec<String>>>,
    domains: Arc<Mutex<HashMap<String, ()>>>,
}

impl MemoryQueue {
    pub fn new(seeds: Vec<String>) -> MemoryQueue {
        MemoryQueue {
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
}

impl Queue for MemoryQueue {
    fn extend(&self, extend: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        for new in extend {
            self.push(new)?;
        }

        Ok(())
    }

    fn drain(&self) -> Result<Drain, Box<dyn std::error::Error>> {
        let mut lock = lock!(self.queue)?;

        match lock.len() {
            0 => Err("empty queue".into()),
            _ => Ok(lock.drain(..).collect::<Vec<String>>()),
        }
    }
}



use std::collections::HashMap;
use std::sync::{Arc, Mutex};

macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().map_err(|_| Into::<Box<dyn std::error::Error>>::into("failed to lock"))
    }
}

pub trait Drain {
    fn len(&self) -> Result<usize, Box<dyn std::error::Error>>;

    fn pop(&self) -> Result<Option<String>, Box<dyn std::error::Error>>;
}

pub trait Queue {
    fn extend(&self, extend: Vec<String>) -> Result<(), Box<dyn std::error::Error>>;

    fn drain(&self) -> Result<Arc<dyn Drain + Send + Sync>, Box<dyn std::error::Error>>;
}

pub struct MemoryQueue {
    queue: Mutex<Vec<String>>,
    domains: Mutex<HashMap<String, ()>>,
}

impl MemoryQueue {
    pub fn new(seeds: Vec<String>) -> MemoryQueue {
        MemoryQueue {
            queue: Mutex::new(seeds),
            domains: Mutex::new(HashMap::new()),
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

    fn drain(&self) -> Result<Arc<dyn Drain + Send + Sync>, Box<dyn std::error::Error>> {
        let mut lock = lock!(self.queue)?;

        let drain = lock.drain(..).collect::<Vec<String>>();

        match drain.len() {
            0 => Err("empty queue".into()),
            _ => Ok(Arc::new(MemoryDrain::new(drain))),
        }
    }
}

pub struct MemoryDrain {
    drain: Mutex<Vec<String>>,
}

impl MemoryDrain {
    pub fn new(drain: Vec<String>) -> MemoryDrain {
        MemoryDrain {
            drain: Mutex::new(drain),
        }
    }
}

impl Drain for MemoryDrain {
    fn len(&self) -> Result<usize, Box<dyn std::error::Error>> {
        lock!(self.drain).map(|drain| drain.len())
    }

    fn pop(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        lock!(self.drain).map(|mut drain| drain.pop())
    }
}



use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Lines, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

macro_rules! lock {
    ($mutex:expr) => {
        $mutex
            .lock()
            .map_err(|_| Into::<Box<dyn std::error::Error>>::into("failed to lock"))
    };
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

pub struct FileQueue {
    queue: Mutex<File>,
    domains: Mutex<HashMap<String, ()>>,
    length: AtomicU64,
}

impl FileQueue {
    pub fn new(seeds: Vec<String>) -> Result<FileQueue, Box<dyn std::error::Error>> {
        let _ = fs::remove_file("queue.ls");

        let mut queue = File::options().append(true).create(true).open("queue.ls")?;

        for seed in seeds.iter().map(|seed| format!("{}\n", seed)) {
            queue.write_all(seed.as_bytes())?;
        }

        Ok(FileQueue {
            queue: Mutex::new(queue),
            domains: Mutex::new(HashMap::new()),
            length: AtomicU64::new(seeds.len() as u64),
        })
    }
}

impl Queue for FileQueue {
    fn extend(&self, extend: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut domains = lock!(self.domains)?;

        let extend = extend
            .iter()
            .filter(|url| {
                let url = url.split('/').take(3).collect::<String>();

                domains.insert(url, ()).is_none()
            })
            .map(|url| format!("{}\n", url).bytes().collect::<Vec<u8>>())
            .collect::<Vec<Vec<u8>>>();

        let bytes = extend.iter().flatten().copied().collect::<Vec<u8>>();

        lock!(self.queue)?.write_all(&bytes)?;

        self.length
            .fetch_add(extend.len() as u64, Ordering::Relaxed);

        Ok(())
    }

    fn drain(&self) -> Result<Arc<dyn Drain + Send + Sync>, Box<dyn std::error::Error>> {
        fs::rename("queue.ls", "drain.ls")?;

        let queue = File::options().append(true).create(true).open("queue.ls")?;

        *lock!(self.queue)? = queue;

        Ok(Arc::new(FileDrain::new(
            self.length.swap(0, Ordering::Relaxed) as usize,
        )?))
    }
}

pub struct FileDrain {
    drain: Mutex<Lines<BufReader<File>>>,
    length: usize,
}

impl FileDrain {
    pub fn new(length: usize) -> Result<FileDrain, Box<dyn std::error::Error>> {
        let drain = BufReader::new(File::open("drain.ls")?);

        Ok(FileDrain {
            drain: Mutex::new(drain.lines()),
            length,
        })
    }
}

impl Drain for FileDrain {
    fn len(&self) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(self.length)
    }

    fn pop(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        lock!(self.drain).map(|mut drain| drain.next().map(|x| x.unwrap_or_default()))
    }
}

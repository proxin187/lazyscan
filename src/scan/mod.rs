mod apache;

use crate::config::{Config, TargetOptions};

use apache::Apache;


pub struct Version {
    epochs: Vec<usize>,
}

impl Version {
    pub fn new(epochs: Vec<usize>) -> Version {
        Version {
            epochs,
        }
    }

    pub fn parse(version: String) -> Version {
        Version {
            epochs: version.split('.')
                .filter_map(|epoch| epoch.parse::<usize>().ok())
                .collect::<Vec<usize>>(),
        }
    }
}

pub trait Target {
    fn verify(&self, url: &str) -> bool;
}

pub struct Scanner {
    targets: Vec<Box<dyn Target>>,
}

impl Scanner {
    pub fn new(config: Config) -> Scanner {
        let targets = config.target.iter()
            .filter_map(|(name, options)| target(&name, options))
            .collect::<Vec<Box<dyn Target>>>();

        Scanner {
            targets,
        }
    }
}

#[inline]
fn target(name: &str, options: &TargetOptions) -> Option<Box<dyn Target>> {
    match name {
        "apache" => None,
        _ => None,
    }
}



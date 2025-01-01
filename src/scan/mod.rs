mod target;

use target::Target;

use crate::config::{Config, TargetOptions};

use reqwest::header::{self, HeaderMap};

use std::ops::Range;


#[derive(Debug)]
pub enum Epoch {
    Range(Range<usize>),
    Point(usize),
}

impl Epoch {
    fn new(epoch: &str) -> Epoch {
        let parts = epoch.split('-')
            .filter_map(|part| part.parse::<usize>().ok())
            .collect::<Vec<usize>>();

        match parts.as_slice() {
            [point] => Epoch::Point(*point),
            [a, b] | [a, .., b] => Epoch::Range(*a..*b),
            _ => unreachable!(),
        }
    }

    fn point(&self) -> Option<usize> {
        match self {
            Epoch::Point(point) => Some(*point),
            Epoch::Range(_) => None,
        }
    }

    fn contains(&self, other: &usize) -> bool {
        match self {
            Epoch::Range(range) => range.contains(other),
            Epoch::Point(point) => point == other,
        }
    }
}

#[derive(Debug)]
pub struct Version {
    epochs: Vec<Epoch>,
}

impl Version {
    pub fn parse(version: &str) -> Version {
        Version {
            epochs: version.split('.')
                .map(|epoch| Epoch::new(epoch))
                .collect::<Vec<Epoch>>(),
        }
    }

    pub fn contains(&self, other: &Version) -> bool {
        self.epochs.len() == other.epochs.len()
            && self.epochs.iter()
                .zip(other.epochs.iter().filter_map(|epoch| epoch.point()))
                .all(|(a, b)| a.contains(&b))
    }
}

pub struct Scanner {
    targets: Vec<Target>,
}

impl Scanner {
    pub fn new(config: &Config) -> Scanner {
        let targets = config.target.iter()
            .filter_map(|(name, options)| target(&name, options))
            .collect::<Vec<Target>>();

        Scanner {
            targets,
        }
    }

    pub fn scan(&self, url: &str, headers: &HeaderMap) {
        for target in self.targets.iter() {
            target.verify(url, headers);
        }
    }
}



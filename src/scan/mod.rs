mod apache;
mod nginx;

use crate::config::{Config, TargetOptions};

use reqwest::header::{self, HeaderMap};

use apache::Apache;
use nginx::Nginx;

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

// TODO: maybe we can represent this as a struct instead, the kind can just be a enum
pub trait Target {
    fn generic(&self, headers: &HeaderMap) -> Option<Version> {
        headers.get(header::SERVER)
            .and_then(|value| {
                value.to_str()
                    .ok()
                    .and_then(|value| value.strip_prefix(&format!("{}/", self.name())))
                    .and_then(|value| value.split(' ').next())
            })
            .map(|value| Version::parse(value))
    }

    fn verify(&self, url: &str, headers: &HeaderMap);

    fn name(&self) -> String;
}

pub struct Scanner {
    targets: Vec<Box<dyn Target + Send + Sync>>,
}

impl Scanner {
    pub fn new(config: &Config) -> Scanner {
        let targets = config.target.iter()
            .filter_map(|(name, options)| target(&name, options))
            .collect::<Vec<Box<dyn Target + Send + Sync>>>();

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

#[inline]
fn target(name: &str, options: &TargetOptions) -> Option<Box<dyn Target + Send + Sync>> {
    match name {
        "apache" => Some(Box::new(Apache::new(options))),
        "nginx" => Some(Box::new(Nginx::new(options))),
        _ => None,
    }
}



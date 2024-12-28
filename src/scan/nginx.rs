use super::{Target, Version, TargetOptions};

use reqwest::header::HeaderMap;


pub struct Nginx {
    version: Version,
    misconfig: bool,
}

impl Nginx {
    pub fn new(options: &TargetOptions) -> Nginx {
        Nginx {
            version: Version::parse(&options.version),
            misconfig: options.misconfig,
        }
    }
}

impl Target for Nginx {
    fn verify(&self, url: &str, headers: &HeaderMap) {
        match self.generic(headers) {
            Some(version) => {
                if self.version.contains(&version) {
                    // println!("url: {}, version: {:?}", url, version);
                }
            },
            None => {},
        }
    }

    fn name(&self) -> String {
        String::from("nginx")
    }
}



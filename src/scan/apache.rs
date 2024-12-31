use super::{Target, Version, TargetOptions};

use reqwest::header::HeaderMap;


pub struct Apache {
    version: Version,
    modules: Vec<String>,
}

impl Apache {
    pub fn new(options: &TargetOptions) -> Apache {
        Apache {
            version: Version::parse(&options.version),
            modules: options.modules.clone(),
        }
    }

    fn detect(&self) {
        for module in self.modules.iter() {
        }
    }
}

impl Target for Apache {
    fn verify(&self, url: &str, headers: &HeaderMap) {
        match self.generic(headers) {
            Some(version) => {
                if self.version.contains(&version) {
                    println!("url: {}, version: {:?}", url, version);
                }
            },
            None => {},
        }
    }

    fn name(&self) -> String {
        String::from("Apache")
    }
}



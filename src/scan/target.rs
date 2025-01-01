use super::{Version, TargetOptions};

use reqwest::header::{self, HeaderMap};

use std::process::Command;


pub struct Target {
    version: Version,
    modules: Vec<String>,
    server: String,
}

impl Target {
    pub fn new(name: String, options: &TargetOptions) -> Target {
        Target {
            version: Version::parse(&options.version),
            modules: options.modules.clone(),
            server: server(name),
        }
    }

    pub fn modules(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        for module in self.modules.iter() {
            let status = Command::new(format!("modules/{}/{}.py", self.server.to_lowercase(), module))
                .arg(url)
                .status()?;

            if status.success() {
                // TODO: we need to log here
            }
        }

        Ok(())
    }

    pub fn scan(&self, headers: &HeaderMap) -> bool {
        let version = headers.get(header::SERVER)
            .and_then(|value| {
                value.to_str()
                    .ok()
                    .and_then(|value| value.strip_prefix(&format!("{}/", self.server)))
                    .and_then(|value| value.split(' ').next())
            })
            .map(|value| Version::parse(value));

        version.map(|version| self.version.contains(&version)).unwrap_or(false)
    }
}

#[inline]
fn server(name: String) -> String {
    match name.to_lowercase().as_str() {
        "apache" => String::from("Apache"),
        "nginx" => String::from("nginx"),
        _ => name,
    }
}



use log::info;

use std::process::Command;


pub struct Scanner {
    modules: Vec<String>,
}

impl Scanner {
    pub fn new(modules: Vec<String>) -> Scanner {
        Scanner {
            modules,
        }
    }

    pub fn modules(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("running modules on {}", url);

        for module in self.modules.iter() {
            let status = Command::new("python")
                .args([
                    format!("modules/{}.py", module),
                    url.to_string(),
                ])
                .status()?;

            if status.code() == Some(0) {
                info!("module success: {}", module);
            }
        }

        Ok(())
    }
}

use super::{Target, Version};


pub struct Apache {
    version: Version,
}

impl Apache {
    pub fn new(version: Version) -> Apache {
        Apache {
            version,
        }
    }
}

impl Target for Apache {
    fn verify(&self, url: &str) -> bool {
        false
    }
}



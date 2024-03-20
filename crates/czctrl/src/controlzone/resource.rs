use anyhow::Ok;
use serde::{Deserialize, Serialize};

use super::util::parse_cpuset;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    pub cpuset: String,
    pub memory: u32,

    #[serde(skip)]
    pub cpus: Vec<u32>,
}

impl Resource {
    pub fn gen_cpus(&mut self) {
        self.cpus = parse_cpuset(&self.cpuset).into_iter().collect();
    }

    pub fn update(&mut self, new: Self) -> anyhow::Result<()> {
        self.cpuset = new.cpuset;
        self.cpus = new.cpus;
        self.memory = new.memory;
        Ok(())
    }
}

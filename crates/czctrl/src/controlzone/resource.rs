use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    pub cpuset: String,
    pub memory: u32,

    #[serde(skip)]
    pub cpus: Vec<u32>,
}

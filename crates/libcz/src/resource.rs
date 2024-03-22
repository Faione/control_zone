use anyhow::{bail, Ok};
use serde::{Deserialize, Serialize};

use super::util::parse_cpuset;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct StaticNet {
    pub address: String,
    pub netmask: String,
    pub gateway: String,
}

impl StaticNet {
    pub fn to_interface_cfg(&self) -> anyhow::Result<String> {
        if self.address == "" || self.netmask == "" || self.gateway == "" {
            bail!("invalid static net config: {:?}", self);
        }

        Ok(format!(
            "iface lo inet loopback
iface eth0 inet static
    address {}
    netmask {}
    gateway {}",
            self.address, self.netmask, self.gateway
        ))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    pub cpuset: String,
    pub memory: u32,
    pub static_net: Option<StaticNet>,

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
        self.static_net = new.static_net;
        Ok(())
    }
}

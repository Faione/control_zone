use anyhow::Ok;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CZOS {
    pub kernel: String,
    pub initram_fs: Option<String>,
    pub rootfs: String,
    pub kcmdline: String,
}

impl CZOS {
    pub fn update(&mut self, new_os: Self) -> anyhow::Result<()> {
        // currentliy not support rootfs update
        self.kernel = new_os.kernel;
        self.kcmdline = new_os.kcmdline;
        Ok(())
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CZOS {
    pub kernel: String,
    pub initram_fs: Option<String>,
    pub rootfs: String,
    pub kcmdline: String,
}

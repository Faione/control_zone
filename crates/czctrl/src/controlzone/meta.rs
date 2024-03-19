use std::path::PathBuf;

use anyhow::{anyhow, Ok};
use serde::{Deserialize, Serialize};

use crate::config::CZ_CONFIG;

use super::default_workdir;

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Meta {
    pub name: String,

    #[serde(default)]
    pub workdir: String,

    #[serde(default)]
    pub share_folder: String,

    #[serde(default)]
    pub full_config: String,
}

impl Meta {
    pub fn is_valid(&self) -> bool {
        return self.workdir != "" && self.share_folder != "" && self.full_config != "";
    }
}

pub struct MetaBuilder {
    meta: Meta,
    workdir: PathBuf,
}

impl MetaBuilder {
    pub fn new(mut meta: Meta, file: &PathBuf) -> anyhow::Result<Self> {
        if meta.name == "" {
            meta.name = file
                .file_name()
                .expect("not a valid file name")
                .to_str()
                .expect("filename convert to str failed")
                .to_owned();
        }

        if meta.workdir == "" {
            meta.workdir = default_workdir(&meta.name)
                .to_str()
                .ok_or(anyhow!("parse workdir failed"))?
                .to_owned();
        }

        let workdir = PathBuf::from(&meta.workdir);

        Ok(MetaBuilder { meta, workdir })
    }

    pub fn with_share_folder(mut self) -> anyhow::Result<Self> {
        if self.meta.share_folder == "" {
            self.meta.share_folder = self
                .workdir
                .join(PathBuf::from("controlzone"))
                .to_str()
                .ok_or(anyhow!("parse workdir failed"))?
                .to_owned();
        }

        Ok(self)
    }

    pub fn with_full_config(mut self) -> anyhow::Result<Self> {
        if self.meta.full_config == "" {
            self.meta.full_config = self
                .workdir
                .join(PathBuf::from(CZ_CONFIG))
                .to_str()
                .ok_or(anyhow!("parse workdir failed"))?
                .to_owned();
        }

        Ok(self)
    }

    pub fn build(self) -> anyhow::Result<Meta> {
        Ok(self.meta)
    }
}

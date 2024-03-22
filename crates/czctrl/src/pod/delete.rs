use std::{fs, path::PathBuf};

use anyhow::{anyhow, bail, Ok, Result};
use clap::Parser;
use libcz::{ControlZone, State, CZ_CONFIG, POD_APPLY_DIR, POD_DIR, POD_DOWN_DIR};

use crate::GloablOpts;

#[derive(Parser, Debug)]
pub struct Delete {
    /// Name of  Control Zone
    #[arg(short, long, required = true)]
    zone: String,

    yaml: PathBuf,
}

pub fn delete(args: Delete, global_opts: &GloablOpts) -> Result<()> {
    if !args.yaml.exists() {
        bail!("not a valid pod yaml")
    }

    let full_config = global_opts.root_dir().join(args.zone).join(CZ_CONFIG);
    let cz = ControlZone::new_from_full_config(&full_config)
        .map_err(|e| anyhow!("error parsing config {:#?}: {}", full_config, e))?;

    if cz.state != State::Running {
        bail!("contol zone {} unable to create pod", cz.meta.name);
    }

    let pod_apply_dir = PathBuf::from(&cz.meta.share_folder)
        .join(POD_DIR)
        .join(POD_APPLY_DIR);

    let Some(yaml_name) = args.yaml.file_name().and_then(|fname| fname.to_str()) else {
        bail!("parse yaml name failed");
    };

    let src_yaml = pod_apply_dir.join(yaml_name);
    if !src_yaml.exists() {
        bail!("pod yaml not applied: {:?}", args.yaml);
    }

    let des_yaml = PathBuf::from(&cz.meta.share_folder)
        .join(POD_DIR)
        .join(POD_DOWN_DIR)
        .join(yaml_name);

    fs::rename(src_yaml, des_yaml)?;
    Ok(())
}

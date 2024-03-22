use std::{fs, path::PathBuf};

use anyhow::{anyhow, bail, Result};
use clap::Parser;
use libcz::{ControlZone, State, CZ_CONFIG, POD_APPLY_DIR, POD_DIR};

use crate::GloablOpts;

#[derive(Parser, Debug)]
pub struct Add {
    /// Name of  Control Zone
    #[arg(short, long, required = true)]
    zone: String,

    yaml: PathBuf,
}

pub fn add(args: Add, global_opts: &GloablOpts) -> Result<()> {
    if !args.yaml.exists() {
        bail!("not a valid pod yaml")
    }

    let full_config = global_opts.root_dir().join(args.zone).join(CZ_CONFIG);
    let cz = ControlZone::new_from_full_config(&full_config)
        .map_err(|e| anyhow!("error parsing config {:#?}: {}", full_config, e))?;

    if cz.state != State::Running {
        bail!("contol zone {} unable to create pod", cz.meta.name);
    }

    let pod_apply_dir = PathBuf::from(cz.meta.share_folder)
        .join(POD_DIR)
        .join(POD_APPLY_DIR);

    let Some(yaml_name) = args.yaml.file_name().and_then(|fname| fname.to_str()) else {
        bail!("parse yaml name failed");
    };

    let des_yaml = pod_apply_dir.join(yaml_name);
    fs::copy(args.yaml, des_yaml)?;

    Ok(())
}

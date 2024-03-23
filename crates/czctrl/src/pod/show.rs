use std::{fs, path::PathBuf};

use anyhow::{anyhow, bail, Ok, Result};
use clap::Parser;
use libcz::{state::State, ControlZone, CZ_CONFIG, POD_APPLY_DIR, POD_DIR};

use crate::GloablOpts;

#[derive(Parser, Debug)]
pub struct Show {
    /// Name of  Control Zone
    #[arg(short, long, required = true)]
    zone: String,
}

pub fn show(args: Show, global_opts: &GloablOpts) -> Result<()> {
    let full_config = global_opts.root_dir().join(args.zone).join(CZ_CONFIG);
    let cz = ControlZone::new_from_full_config(&full_config)
        .map_err(|e| anyhow!("error parsing config {:#?}: {}", full_config, e))?;

    if cz.state != State::Running {
        bail!("contol zone {} unable to create pod", cz.meta.name);
    }

    let pod_apply_dir = PathBuf::from(cz.meta.share_folder)
        .join(POD_DIR)
        .join(POD_APPLY_DIR);

    fs::read_dir(pod_apply_dir)?
        .filter_map(|entry| entry.ok())
        .for_each(|entry| println!("{:?}", entry.file_name()));

    Ok(())
}

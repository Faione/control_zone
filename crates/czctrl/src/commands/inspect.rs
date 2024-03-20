use std::path::PathBuf;

use anyhow::{anyhow, Ok, Result};
use clap::Parser;

use crate::GloablOpts;

use libcz::{default_workdir, ControlZone, CZ_CONFIG};

#[derive(Parser, Debug)]
pub struct Inspect {
    /// Control Zone Config
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Name of Control Zone
    control_zone: String,
}

pub fn inspect(args: Inspect, _: &GloablOpts) -> Result<()> {
    let full_config = match args.config {
        Some(path) => path,
        None => default_workdir(&args.control_zone).join(CZ_CONFIG),
    };

    let cz = ControlZone::new_from_full_config(&full_config)
        .map_err(|e| anyhow!("error parsing config {:#?}: {}", full_config, e))?;
    serde_yaml::to_string(&cz).map(|cz_str| println!("{}", cz_str))?;
    Ok(())
}

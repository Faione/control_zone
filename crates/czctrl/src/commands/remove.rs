use std::path::PathBuf;

use anyhow::{anyhow, bail, Ok, Result};
use clap::Parser;
use log::info;

use crate::{
    commands::stop::{self, stop_inner, Stop},
    config::CZ_CONFIG,
    controlzone::{self, default_workdir},
    GloablOpts,
};

#[derive(Parser, Debug)]
pub struct Remove {
    /// Force to remove
    #[arg(short, long)]
    force: bool,

    /// Control Zone Config
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Name of Control Zone
    control_zone: String,
}

pub fn remove(args: Remove, global_opts: &GloablOpts) -> Result<()> {
    let full_config = match args.config {
        Some(path) => path,
        None => default_workdir(&args.control_zone).join(CZ_CONFIG),
    };

    let mut cz = controlzone::ControlZone::new_from_full_config(&full_config)
        .map_err(|e| anyhow!("error parsing config {:#?}: {}", full_config, e))?;

    if global_opts.dry_run {
        return Ok(());
    }

    if args.force {
        stop_inner(&mut cz)?
    }

    if let Err(e) = cz.remove() {
        bail!("remove control zone failed: {e}")
    }

    info!("{} removed", cz.meta.name);
    Ok(())
}

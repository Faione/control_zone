use std::path::PathBuf;

use anyhow::{anyhow, bail, Ok, Result};
use clap::Parser;
use log::info;

use crate::{vruntime::VRuntime, GloablOpts};

use libcz::{default_workdir, ControlZone, CZ_CONFIG};

#[derive(Parser, Debug)]
pub struct Stop {
    /// Control Zone Config
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Name of Control Zone
    control_zone: String,
}

pub fn stop(args: Stop, global_opts: &GloablOpts) -> Result<()> {
    let full_config = match args.config {
        Some(path) => path,
        None => default_workdir(&args.control_zone).join(CZ_CONFIG),
    };

    let mut cz = ControlZone::new_from_full_config(&full_config)
        .map_err(|e| anyhow!("error parsing config {:#?}: {}", full_config, e))?;

    if global_opts.dry_run {
        return Ok(());
    }

    let vruntime: VRuntime = global_opts.vruntime.into();
    stop_inner(&mut cz, &vruntime)
}

pub fn stop_inner(cz: &mut ControlZone, vruntime: &VRuntime) -> Result<()> {
    if let Err(e) = cz.stop(&vruntime.stop_f) {
        bail!("stop {} failed: {e}", cz.meta.name)
    }

    info!("{} stopped", cz.meta.name);
    Ok(())
}

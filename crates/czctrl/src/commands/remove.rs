use std::path::PathBuf;

use anyhow::{anyhow, bail, Ok, Result};
use clap::Parser;
use log::info;

use crate::{commands::stop::stop_inner, GloablOpts};

use libcz::{default_workdir, state::State, vruntime::DVRuntime, ControlZone, CZ_CONFIG};

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

    let mut cz = ControlZone::new_from_full_config(&full_config)
        .map_err(|e| anyhow!("error parsing config {:#?}: {}", full_config, e))?;

    if global_opts.dry_run {
        return Ok(());
    }

    let vruntime: DVRuntime = global_opts.vruntime.into();
    remove_inner(&mut cz, args.force, &vruntime)
}

pub fn remove_inner(cz: &mut ControlZone, force: bool, vruntime: &DVRuntime) -> Result<()> {
    if cz.state == State::Running && force {
        stop_inner(cz, vruntime)?
    }

    if let Err(e) = cz.remove() {
        bail!("remove control zone failed: {e}")
    }

    info!("{} removed", cz.meta.name);
    Ok(())
}

use std::path::PathBuf;

use anyhow::{anyhow, Ok, Result};
use clap::Parser;
use log::{debug, info};

use crate::{
    commands::{start::start_inner, stop::stop_inner},
    GloablOpts,
};

use libcz::{default_workdir, ControlZone, UpdateMode, CZ_CONFIG};

#[derive(Parser, Debug)]
pub struct Update {
    /// Wait untail Vm Boot
    #[arg(short, long)]
    wait: bool,

    /// Control Zone Config
    #[arg(short, long, required = true)]
    file: PathBuf,

    /// Name of Control Zone
    control_zone: String,
}

pub fn update(args: Update, global_opts: &GloablOpts) -> Result<()> {
    // current controlzone
    let full_config = default_workdir(&args.control_zone).join(CZ_CONFIG);
    let mut curr_cz = ControlZone::new_from_full_config(&full_config)
        .map_err(|e| anyhow!("error parsing config {:#?}: {}", full_config, e))?;

    // new controlzone
    let new_cz = ControlZone::new_from_config(&args.file)?;
    if global_opts.dry_run {
        return Ok(());
    }

    update_innner(&mut curr_cz, new_cz, args.wait)
}

pub fn update_innner(curr_cz: &mut ControlZone, new_cz: ControlZone, wait: bool) -> Result<()> {
    let update_mod = curr_cz.update_config(new_cz)?;
    debug!("control zone update mode: {:?}", update_mod);
    match update_mod {
        UpdateMode::Reboot => {
            stop_inner(curr_cz)?;
            start_inner(curr_cz, wait)?;

            info!("control zone {} have updated", curr_cz.meta.name);
            Ok(())
        }
        UpdateMode::Hot => todo!(),
        UpdateMode::Stale => {
            info!("control zone {} have not been changed", curr_cz.meta.name);
            Ok(())
        }
    }
}

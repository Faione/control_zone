use std::path::PathBuf;

use anyhow::{anyhow, bail, Ok, Result};
use clap::Parser;
use log::info;

use crate::{
    config::{CZ_CONFIG, DEFAUL_LIBVIRT_URI},
    controlzone::{self, default_workdir, ControlZone},
    GloablOpts,
};

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

    let mut cz = controlzone::ControlZone::new_from_full_config(&full_config)
        .map_err(|e| anyhow!("error parsing config {:#?}: {}", full_config, e))?;

    if global_opts.dry_run {
        return Ok(());
    }

    stop_inner(&mut cz)
}

pub fn stop_inner(cz: &mut ControlZone) -> Result<()> {
    let libvirt_stop_f = |controlzone: &ControlZone| -> anyhow::Result<()> {
        let virt_cli = libvm::virt::Libvirt::connect(DEFAUL_LIBVIRT_URI)?;
        let cz_wrapper = virt_cli.get_control_zone_by_name(&controlzone.meta.name)?;
        cz_wrapper.destroy()
    };

    if let Err(e) = cz.stop(libvirt_stop_f) {
        bail!("stop {} failed: {e}", cz.meta.name)
    }

    info!("{} stopped", cz.meta.name);
    Ok(())
}

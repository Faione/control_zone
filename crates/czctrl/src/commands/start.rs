use std::path::PathBuf;

use anyhow::{anyhow, bail, Ok, Result};
use clap::Parser;
use log::info;

use crate::{vruntime::VRuntime, GloablOpts};

use libcz::{default_workdir, ControlZone, CZ_CONFIG};

#[derive(Parser, Debug)]
pub struct Start {
    /// Control Zone Config
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Wait untail Vm Boot
    #[arg(short, long)]
    wait: bool,

    /// asign static ip and using bridge network
    #[arg(short, long)]
    ip: Option<String>,

    /// Name of Control Zone
    control_zone: String,
}

pub fn start(args: Start, global_opts: &GloablOpts) -> Result<()> {
    let full_config = match args.config {
        Some(path) => path,
        None => default_workdir(&args.control_zone).join(CZ_CONFIG),
    };

    let mut cz = ControlZone::new_from_full_config(&full_config)
        .map_err(|e| anyhow!("error parsing config {:#?}: {}", full_config, e))?;

    if global_opts.dry_run {
        println!("{}", cz.to_xml()?);
        return Ok(());
    }

    let vruntime: VRuntime = global_opts.vruntime.into();
    start_inner(&mut cz, args.wait, &vruntime)
}

pub fn start_inner(cz: &mut ControlZone, wait: bool, vruntime: &VRuntime) -> Result<()> {
    let wf_op = if wait { Some(&vruntime.wait_f) } else { None };
    info!("starting controlzone...");
    if let Err(e) = cz.start(&vruntime.start_f, wf_op) {
        bail!("start {} failed: {e}", cz.meta.name)
    }

    info!("{} started", cz.meta.name);
    Ok(())
}

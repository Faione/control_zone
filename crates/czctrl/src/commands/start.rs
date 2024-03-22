use std::{path::PathBuf, sync::mpsc, time::Duration};

use anyhow::{anyhow, bail, Ok, Result};
use clap::Parser;
use log::{debug, error, info};
use notify::{
    event::{AccessKind, AccessMode},
    Watcher,
};

use crate::{
    config::{DEFAUL_LIBVIRT_URI, TRY_COUNT, TRY_INTERVAL},
    GloablOpts,
};

use libcz::{default_workdir, ControlZone, State, CZ_CONFIG};

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

    start_inner(&mut cz, args.wait)
}

pub fn start_inner(cz: &mut ControlZone, wait: bool) -> Result<()> {
    let libvirt_start_f = |controlzone: &ControlZone| -> anyhow::Result<()> {
        let virt_cli = libvm::virt::Libvirt::connect(DEFAUL_LIBVIRT_URI)?;
        virt_cli.create_control_zone(&controlzone.to_xml()?)?;

        Ok(())
    };

    let wait_f = |state_f: &PathBuf| -> anyhow::Result<State> {
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(
            move |res: std::prelude::v1::Result<notify::Event, notify::Error>| match res {
                Result::Ok(event) => {
                    debug!("{:?}", event);
                    match event.kind {
                        notify::EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                            debug!("control zone state modified",);
                            if let Err(e) = tx.send({}) {
                                error!("failed to notify file change: {e}");
                            }
                        }
                        _ => {}
                    }
                }
                Err(_) => {}
            },
        )?;

        watcher.watch(state_f, notify::RecursiveMode::NonRecursive)?;
        rx.recv_timeout(Duration::from_secs((TRY_INTERVAL * TRY_COUNT) as u64))?;
        watcher.unwatch(state_f)?;
        debug!("stop watch state");

        // TODO: check state and avoid race
        Ok(State::Running)
    };

    let wf_op = if wait { Some(wait_f) } else { None };

    info!("starting controlzone...");
    if let Err(e) = cz.start(libvirt_start_f, wf_op) {
        bail!("start {} failed: {e}", cz.meta.name)
    }

    info!("{} started", cz.meta.name);
    Ok(())
}

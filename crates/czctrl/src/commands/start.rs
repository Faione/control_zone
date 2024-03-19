use std::{path::PathBuf, thread::sleep, time::Duration};

use anyhow::{anyhow, bail, Ok, Result};
use clap::Parser;
use log::{debug, info};

use crate::{
    config::{CZ_CONFIG, DEFAUL_LIBVIRT_URI, TRY_COUNT, TRY_INTERVAL},
    controlzone::{self, default_workdir, ControlZone},
    GloablOpts,
};

#[derive(Parser, Debug)]
pub struct Start {
    /// Control Zone Config
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Wait untail Vm Boot
    #[arg(short, long)]
    wait: bool,

    /// Name of Control Zone
    control_zone: String,
}

pub fn start(args: Start, global_opts: &GloablOpts) -> Result<()> {
    let full_config = match args.config {
        Some(path) => path,
        None => default_workdir(&args.control_zone).join(CZ_CONFIG),
    };

    let mut cz = controlzone::ControlZone::new_from_full_config(&full_config)
        .map_err(|e| anyhow!("error parsing config {:#?}: {}", full_config, e))?;

    if global_opts.dry_run {
        println!("{}", cz.to_xml()?);
        return Ok(());
    }

    let libvirt_start_f = |controlzone: &ControlZone| -> anyhow::Result<()> {
        let virt_cli = libvm::virt::Libvirt::connect(DEFAUL_LIBVIRT_URI)?;
        let cz_wrapper = virt_cli.create_control_zone(&controlzone.to_xml()?)?;

        if args.wait {
            let mut try_count = TRY_COUNT;
            let ip = loop {
                match cz_wrapper.get_ip() {
                    Result::Ok(ip) => break Ok(ip),
                    Err(e) => {
                        if try_count > 0 {
                            debug!("try ip detecting: {try_count}...");
                            sleep(Duration::from_secs(TRY_INTERVAL));
                            try_count -= 1;
                            continue;
                        }

                        break Err(anyhow!("{e}"));
                    }
                }
            }?;

            info!("{} started, ip: {}", controlzone.meta.name, ip);
        }

        Ok(())
    };

    if let Err(e) = cz.start(libvirt_start_f) {
        bail!("start {} failed: {e}", cz.meta.name)
    }

    info!("{} started", cz.meta.name);
    Ok(())
}

use std::{path::PathBuf, thread::sleep, time::Duration};

use anyhow::{anyhow, Ok, Result};
use clap::Parser;
use log::{debug, error, info};

use crate::{
    config::{DEFAUL_LIBVIRT_URI, TRY_COUNT, TRY_INTERVAL},
    controlzone,
};

#[derive(Parser, Debug)]
pub struct Apply {
    /// Control Zone Config
    #[arg(short, long, required = true)]
    file: PathBuf,
}

pub fn apply(args: Apply) -> Result<()> {
    let virt_cli = libvm::virt::Libvirt::connect(DEFAUL_LIBVIRT_URI)?;

    let mut cz = controlzone::ControlZone::new_from_config(&args.file)?;
    if let Err(e) = cz.init_workdir() {
        error!("init control zone workdir failed: {e}")
    }

    debug!("workdir initialized\n{:#?}", cz);

    let xml = cz.to_xml()?;
    let cz_wrapper = virt_cli.create_control_zone(&xml)?;

    info!("{} created...", cz.meta.name);

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

    info!("{} initialized: {}", cz.meta.name, ip);
    Ok(())
}

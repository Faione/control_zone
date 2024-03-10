use std::{fs, path::PathBuf, thread::sleep, time::Duration};

use anyhow::{anyhow, Ok, Result};
use clap::Parser;

use crate::{commands::DEFAUL_LIBVIRT_URI, control_zone::ControlZone};

#[derive(Parser, Debug)]
pub struct Apply {
    /// Control Zone Config
    #[arg(short, long)]
    file: PathBuf,
}

pub fn apply(args: Apply) -> Result<()> {
    let virt_cli = libvm::virt::Libvirt::connect(DEFAUL_LIBVIRT_URI)?;

    let config = fs::read_to_string(args.file)?;
    let cz: ControlZone = serde_yaml::from_str(&config)?;
    let xml = cz.to_xml()?;

    let cz_wrapper = virt_cli.create_control_zone(&xml)?;

    let mut try_count = 5;
    let ip = loop {
        match cz_wrapper.get_ip() {
            Result::Ok(ip) => break Ok(ip),
            Err(e) => {
                if try_count > 0 {
                    println!("left {try_count} tries");
                    sleep(Duration::from_secs(5));
                    try_count -= 1;
                    continue;
                }

                break Err(anyhow!("{e}"));
            }
        }
    }?;

    println!("control zone ip: {ip}");
    Ok(())
}

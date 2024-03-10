use anyhow::Result;
use clap::Parser;
use libvm::virt;

use super::DEFAUL_LIBVIRT_URI;

#[derive(Parser, Debug)]
pub struct List {}

pub fn list(args: List) -> Result<()> {
    let virt_cli = virt::Libvirt::connect(DEFAUL_LIBVIRT_URI)?;
    let control_zone_wps = virt_cli.get_control_zone_wrappers()?;

    for cz in control_zone_wps {
        println!("cz ip {}", cz.get_ip()?);
    }

    Ok(())
}

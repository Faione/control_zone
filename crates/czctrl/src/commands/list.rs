use anyhow::Result;
use clap::Parser;
use libvm::virt;

use super::DEFAUL_LIBVIRT_URI;

#[derive(Parser, Debug)]
pub struct List {
    /// List all Control Zones
    #[arg(short, long)]
    all: bool,
}

pub fn list(args: List) -> Result<()> {
    let virt_cli = virt::Libvirt::connect(DEFAUL_LIBVIRT_URI)?;
    let control_zone_wps = virt_cli.get_control_zone_wrappers()?;

    if args.all {
        for cz in control_zone_wps {
            let ip = cz.get_ip()?;
            let id = cz.get_id()?;
            let name = cz.get_name()?;

            println!("{:6} {:20} {:}", id, name, ip);
        }
    }

    Ok(())
}

use anyhow::Result;
use clap::Parser;

use super::DEFAUL_LIBVIRT_URI;

#[derive(Parser, Debug)]
pub struct Down {
    /// Control Zone id
    id: u32,
}

pub fn down(args: Down) -> Result<()> {
    let virt_cli = libvm::virt::Libvirt::connect(DEFAUL_LIBVIRT_URI)?;
    let cz_wrapper = virt_cli.get_control_zone_by_id(args.id)?;
    cz_wrapper.destroy()?;
    Ok(())
}

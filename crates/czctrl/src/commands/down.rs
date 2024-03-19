use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use log::error;

use crate::{config::DEFAUL_LIBVIRT_URI, controlzone};

#[derive(Parser, Debug)]
pub struct Down {
    /// Control Zone Config
    #[arg(short, long, required = true)]
    file: PathBuf,
}

pub fn down(args: Down) -> Result<()> {
    let virt_cli = libvm::virt::Libvirt::connect(DEFAUL_LIBVIRT_URI)?;

    let cz = controlzone::ControlZone::new_from_config(&args.file)?;
    let cz_wrapper = virt_cli.get_control_zone_by_name(&cz.meta.name)?;
    if let Err(e) = cz_wrapper.destroy() {
        error!("destroy control zone failed: {e}");
    };

    cz.delete_workdir()?;
    Ok(())
}

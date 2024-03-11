use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use log::error;

use crate::control_zone::ControlZone;

use super::DEFAUL_LIBVIRT_URI;

#[derive(Parser, Debug)]
pub struct Down {
    /// Control Zone Config
    #[arg(short, long, required = true)]
    file: PathBuf,
}

pub fn down(args: Down) -> Result<()> {
    let virt_cli = libvm::virt::Libvirt::connect(DEFAUL_LIBVIRT_URI)?;

    let config = fs::read_to_string(args.file)?;
    let cz: ControlZone = serde_yaml::from_str(&config)?;

    if let Err(e) = cz.delete_workdir() {
        error!("clean control zone workdir failed: {e}");
    }

    let cz_wrapper = virt_cli.get_control_zone_by_name(&cz.name)?;

    if let Err(e) = cz_wrapper.destroy() {
        error!("destroy control zone failed: {e}");
    };
    Ok(())
}

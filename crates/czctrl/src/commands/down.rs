use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::controlzone;

use super::remove::remove_inner;

#[derive(Parser, Debug)]
pub struct Down {
    /// Control Zone Config
    #[arg(short, long, required = true)]
    file: PathBuf,
}

pub fn down(args: Down) -> Result<()> {
    let mut cz = controlzone::ControlZone::new_from_config(&args.file)?;
    remove_inner(&mut cz, true)
}

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::{
    commands::{create::create_inner, start::start_inner},
    controlzone,
};

#[derive(Parser, Debug)]
pub struct Apply {
    /// Wait untail Vm Boot
    #[arg(short, long)]
    wait: bool,

    /// Control Zone Config
    #[arg(short, long, required = true)]
    file: PathBuf,
}

pub fn apply(args: Apply) -> Result<()> {
    let mut cz = controlzone::ControlZone::new_from_config(&args.file)?;
    create_inner(&mut cz)?;
    start_inner(&mut cz, args.wait)
}

use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use log::error;

use crate::control_zone::ControlZone;

#[derive(Parser, Debug)]
pub struct Generate {
    /// Init Control Zone while generate
    #[arg(short, long)]
    init: bool,

    /// Control Zone Config
    #[arg(short, long, required = true)]
    file: PathBuf,
}

pub fn generate(args: Generate) -> Result<()> {
    let config = fs::read_to_string(args.file)?;

    let mut cz: ControlZone = serde_yaml::from_str(&config)?;
    if args.init {
        if let Err(e) = cz.init_workdir() {
            error!("init control zone workdir failed: {e}")
        }
    }

    let xml = cz.to_xml()?;
    println!("{}", xml);
    Ok(())
}

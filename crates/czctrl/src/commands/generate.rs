use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;

use crate::control_zone::ControlZone;

#[derive(Parser, Debug)]
pub struct Generate {
    /// Control Zone Config
    #[arg(short, long)]
    file: PathBuf,
}

pub fn generate(args: Generate) -> Result<()> {
    let config = fs::read_to_string(args.file)?;

    let cz: ControlZone = serde_yaml::from_str(&config)?;

    let xml = cz.to_xml()?;
    println!("{}", xml);
    Ok(())
}

use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Parser;
use libcz::{ControlZone, State};

use crate::commands::{create::create_inner, start::start_inner};

use super::update::update_innner;

#[derive(Parser, Debug)]
pub struct Apply {
    /// Wait untail Vm Boot
    #[arg(short, long)]
    wait: bool,

    /// asign static ip and using bridge network
    #[arg(short, long)]
    ip: Option<String>,

    /// Control Zone Config
    #[arg(short, long, required = true)]
    file: PathBuf,
}

pub fn apply(args: Apply) -> Result<()> {
    let mut new_cz = ControlZone::new_from_config(&args.file)?;

    match new_cz.state {
        State::Pending => {
            create_inner(&mut new_cz)?;
            start_inner(&mut new_cz, args.wait)
        }
        _ => {
            let full_config = PathBuf::from(&new_cz.meta.full_config);
            let mut curr_cz = libcz::ControlZone::new_from_full_config(&full_config)
                .map_err(|e| anyhow!("error parsing config {:#?}: {}", full_config, e))?;

            update_innner(&mut curr_cz, new_cz, args.wait)
        }
    }
}

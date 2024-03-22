use std::{fs, path::PathBuf};

use anyhow::{anyhow, Ok, Result};
use clap::Parser;
use libcz::{ControlZone, CZ_CONFIG, POD_CRUNTIME_LOG, POD_DIR};

use crate::GloablOpts;

#[derive(Parser, Debug)]
pub struct Log {
    /// Name of  Control Zone
    #[arg(short, long, required = true)]
    zone: String,
}

pub fn log(args: Log, global_opts: &GloablOpts) -> Result<()> {
    let full_config = global_opts.root_dir().join(args.zone).join(CZ_CONFIG);

    let cz = ControlZone::new_from_full_config(&full_config)
        .map_err(|e| anyhow!("error parsing config {:#?}: {}", full_config, e))?;

    let log_file = PathBuf::from(cz.meta.share_folder)
        .join(POD_DIR)
        .join(POD_CRUNTIME_LOG);

    println!("{}", fs::read_to_string(log_file)?);
    Ok(())
}

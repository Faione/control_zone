use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;
use log::info;

use crate::{controlzone, GloablOpts};

#[derive(Parser, Debug)]
pub struct Create {
    /// Control Zone Config
    #[arg(short, long, required = true)]
    file: PathBuf,
}

pub fn create(args: Create, global_opts: &GloablOpts) -> Result<()> {
    // 1. check config
    let mut cz = controlzone::ControlZone::new_from_config(&args.file)?;

    if global_opts.dry_run {
        println!("{:#?}", cz);
        return Ok(());
    }

    // 2. valid check
    if cz.test_exists().is_some() {
        bail!(
            "attempting to create on an existing control zone, check your dir: {}",
            cz.meta.workdir
        )
    }

    // 3. create control zone
    if let Err(e) = cz.create() {
        bail!("create control zone failed: {e}")
    }

    info!("{} created", cz.meta.name);
    Ok(())
}

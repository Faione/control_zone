use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;
use log::info;

use crate::GloablOpts;

use libcz::ControlZone;

#[derive(Parser, Debug)]
pub struct Create {
    /// Control Zone Config
    #[arg(short, long, required = true)]
    file: PathBuf,
}

pub fn create(args: Create, global_opts: &GloablOpts) -> Result<()> {
    // check config
    let mut cz = ControlZone::new_from_config(&args.file)?;

    if global_opts.dry_run {
        println!("{:#?}", cz);
        return Ok(());
    }

    create_inner(&mut cz)
}

pub fn create_inner(cz: &mut ControlZone) -> Result<()> {
    // valid check
    if cz.test_exists().is_some() {
        bail!(
            "attempting to create on an existing control zone, check your dir: {}",
            cz.meta.workdir
        )
    }

    // create control zone
    if let Err(e) = cz.create() {
        bail!("create control zone failed: {e}")
    }

    info!("{} created", cz.meta.name);
    Ok(())
}

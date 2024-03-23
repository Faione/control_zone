use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::GloablOpts;

use super::remove::remove_inner;
use libcz::{vruntime::DVRuntime, ControlZone};

#[derive(Parser, Debug)]
pub struct Down {
    /// Control Zone Config
    #[arg(short, long, required = true)]
    file: PathBuf,
}

pub fn down(args: Down, global_opts: &GloablOpts) -> Result<()> {
    let mut cz = ControlZone::new_from_config(&args.file)?;
    let vruntime: DVRuntime = global_opts.vruntime.into();
    remove_inner(&mut cz, true, &vruntime)
}

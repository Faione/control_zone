use anyhow::Result;
use clap::Parser;

use crate::GloablOpts;

#[derive(Parser, Debug)]
pub struct Remove {}

pub fn remove(args: Remove, global_opts: &GloablOpts) -> Result<()> {
    todo!()
}

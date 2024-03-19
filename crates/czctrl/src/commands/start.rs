use anyhow::Result;
use clap::Parser;

use crate::GloablOpts;

#[derive(Parser, Debug)]
pub struct Start {}

pub fn start(args: Start, global_opts: &GloablOpts) -> Result<()> {
    todo!()
}

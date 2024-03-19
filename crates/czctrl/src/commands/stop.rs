use anyhow::Result;
use clap::Parser;

use crate::GloablOpts;

#[derive(Parser, Debug)]
pub struct Stop {}

pub fn stop(args: Stop, global_opts: &GloablOpts) -> Result<()> {
    todo!()
}

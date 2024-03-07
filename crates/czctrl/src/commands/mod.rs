use clap::Parser;

use self::{apply::Apply, down::Down, generate::Generate, observe::Observe};

pub mod apply;
pub mod down;
pub mod generate;
pub mod observe;

#[derive(Parser, Debug)]
pub enum ControlZoneCmd {
    /// Apply Control Zone YAML
    Apply(Apply),

    /// Start Control Zone
    Down(Down),

    /// Generate Control Zone YAML
    Generate(Generate),

    /// Monitor Control Zone
    Observe(Observe),
}

use clap::Parser;

use self::{
    apply::Apply, create::Create, down::Down, list::List, observe::Observe, remove::Remove,
    start::Start, stop::Stop,
};

pub mod apply;

pub mod down;
pub mod list;
pub mod observe;

pub mod create;
pub mod remove;
pub mod start;
pub mod stop;

#[derive(Parser, Debug)]
pub enum AdvanceCmd {
    /// Apply Control Zone YAML
    Apply(Apply),

    /// Start Control Zone
    Down(Down),

    /// List Control Zones
    List(List),

    /// Monitor Control Zone
    Observe(Observe),
}

#[derive(Parser, Debug)]
pub enum BasicCmd {
    /// Create Control Zone
    Create(Create),

    /// Start Created Control Zone
    Start(Start),

    /// Stop Started Control Zone
    Stop(Stop),

    /// Removed Stopped Control Zone
    Remove(Remove),
}

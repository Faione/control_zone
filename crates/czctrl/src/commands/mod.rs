use clap::Parser;

use self::{
    apply::Apply, create::Create, down::Down, list::List, observe::Observe, remove::Remove,
    start::Start, stop::Stop, update::Update,
};

pub mod apply;

pub mod down;
pub mod list;
pub mod observe;

pub mod create;
pub mod remove;
pub mod start;
pub mod stop;
pub mod update;

#[derive(Parser, Debug)]
pub enum AdvanceCmd {
    /// Apply Control Zone from Yaml
    Apply(Apply),

    /// Down Control Zone from Yaml
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

    /// Start Control Zone
    Start(Start),

    /// Update Control ZOne
    Update(Update),

    /// Stop Control Zone
    Stop(Stop),

    /// Removed Control Zone
    Remove(Remove),
}

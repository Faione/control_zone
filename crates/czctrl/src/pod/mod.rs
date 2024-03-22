use clap::Parser;

use self::{add::Add, delete::Delete, show::Show};

pub mod add;
pub mod delete;
pub mod show;

#[derive(Parser, Debug)]
pub enum PodCmd {
    /// Add Pod to Control Zone
    Add(Add),

    /// Delete Pod from Control Zone
    Delete(Delete),

    /// List Pod of Control Zone
    Show(Show),
}

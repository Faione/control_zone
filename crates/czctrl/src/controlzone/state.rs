use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    Pendding,
    Created,
    Running,
    Stopped,
    Error,
}

impl Default for State {
    fn default() -> Self {
        Self::Pendding
    }
}

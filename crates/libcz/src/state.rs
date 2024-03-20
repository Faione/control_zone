use anyhow::{bail, Ok};
use strum::{Display, EnumString};

#[derive(Debug, EnumString, Display, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    Pending,
    Created,
    Running,
    Stopped,
    Zombied,
    Error,
}

impl Default for State {
    fn default() -> Self {
        Self::Pending
    }
}

impl State {
    // result means update check
    // bool means stale
    // state may not be changed and keep stale
    pub fn check_update(&self, new_state: State) -> anyhow::Result<bool> {
        if *self == new_state {
            return Ok(true);
        }

        match (self, new_state) {
            (State::Pending, State::Created) => {}
            (State::Created, State::Running) => {}
            (State::Running, State::Stopped) => {}
            (State::Running, State::Error) => {}
            (State::Stopped, State::Running) => {}
            (State::Stopped, State::Zombied) => {}
            (State::Created, State::Zombied) => {}

            _ => bail!("can not change state from {:#?} to {:#?}", self, new_state),
        }
        Ok(false)
    }
}

use std::{sync::mpsc, time::Duration};

use clap::ValueEnum;
use libcz::{ControlZone, State};
use log::{debug, error};
use notify::{
    event::{AccessKind, AccessMode},
    Watcher,
};

mod libvirt;

const TRY_COUNT: i8 = 10;
const TRY_INTERVAL: i8 = 1; // try interval (second)

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum VRuntimeType {
    Libvirt,
    Qemu,
    CloudHyper,
}

pub struct VRuntime {
    pub start_f: Box<dyn Fn(&ControlZone) -> anyhow::Result<()>>,
    pub stop_f: Box<dyn Fn(&ControlZone) -> anyhow::Result<()>>,
    pub wait_f: Box<dyn Fn(&ControlZone) -> anyhow::Result<State>>,
}

impl From<VRuntimeType> for VRuntime {
    fn from(t: VRuntimeType) -> Self {
        match t {
            VRuntimeType::Libvirt => libvirt::vruntime(),
            VRuntimeType::Qemu => todo!(),
            VRuntimeType::CloudHyper => todo!(),
        }
    }
}

pub fn cz_wait_f(cz: &ControlZone) -> anyhow::Result<State> {
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(
        move |res: std::prelude::v1::Result<notify::Event, notify::Error>| match res {
            Result::Ok(event) => {
                debug!("{:?}", event);
                match event.kind {
                    notify::EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                        debug!("control zone state modified",);
                        if let Err(e) = tx.send({}) {
                            error!("failed to notify file change: {e}");
                        }
                    }
                    _ => {}
                }
            }
            Err(_) => {}
        },
    )?;

    let state_f = cz.state_file();
    watcher.watch(&state_f, notify::RecursiveMode::NonRecursive)?;
    rx.recv_timeout(Duration::from_secs((TRY_INTERVAL * TRY_COUNT) as u64))?;
    watcher.unwatch(&state_f)?;
    debug!("stop watch state");

    // TODO: check state and avoid race
    Ok(State::Running)
}

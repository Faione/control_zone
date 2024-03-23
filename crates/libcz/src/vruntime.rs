use std::{fs, path::PathBuf, sync::mpsc, time::Duration};

use anyhow::Ok;
use log::{debug, error};
use notify::{
    event::{AccessKind, AccessMode},
    Watcher,
};

use crate::{state::State, ControlZone, INFO_DIR, IP_FILE};

const WAIT_TIMEOUT: u64 = 10;

pub type InfoPer = Box<dyn Fn(&ControlZone) -> anyhow::Result<()>>;
pub type DVRuntime = Box<dyn VRuntime>;

#[inline]
pub fn addition_info_bar() {
    println!("{:16}", "IP");
}

pub fn addition_info_per(cz: &ControlZone) -> anyhow::Result<()> {
    let ip_file = PathBuf::from(&cz.meta.share_folder)
        .join(INFO_DIR)
        .join(IP_FILE);

    if !ip_file.exists() {
        error!("control zone may not initialized")
    }

    println!("{}", fs::read_to_string(ip_file)?);
    Ok(())
}

pub trait VRuntime {
    fn start(&self, cz: &mut ControlZone) -> anyhow::Result<()>;
    fn stop(&self, cz: &mut ControlZone) -> anyhow::Result<()>;

    fn addi_bar(&self) {
        addition_info_bar();
    }
    fn addi_infoper(&self, cz: &ControlZone) -> anyhow::Result<()> {
        addition_info_per(cz)
    }

    fn wait(&self, cz: &mut ControlZone) -> anyhow::Result<()> {
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
        rx.recv_timeout(Duration::from_secs(WAIT_TIMEOUT))?;
        watcher.unwatch(&state_f)?;
        debug!("stop watch state");

        // TODO: check state and avoid race
        cz.state = State::Running;
        Ok(())
    }
}

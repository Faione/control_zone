use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};

use log::debug;
use notify::{event::CreateKind, RecursiveMode, Watcher};

use crate::worker::{Event, PodOps};
use libcz::{POD_APPLY_DIR, POD_DOWN_DIR};
#[cfg(not(feature = "poll_watcher"))]
use notify::event::{ModifyKind, RenameMode};
use std::sync::mpsc;

#[cfg(feature = "poll_watcher")]
use notify::event::RemoveKind;

#[cfg(feature = "poll_watcher")]
pub fn watcher_loop(pod_root: PathBuf, tx: mpsc::Sender<Event>) -> anyhow::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    let root = pod_root.clone();
    let event_handler = move |res: Result<notify::Event, notify::Error>| match res {
        Ok(event) => match event.kind {
            notify::EventKind::Create(CreateKind::Any) => {
                let src_file = &event.paths[0];
                let Some(parent) = src_file.parent().and_then(|d| d.file_name()) else {
                    return;
                };

                if parent != POD_APPLY_DIR {
                    return;
                }

                let Some(yaml) = PathBuf::from(&event.paths[0])
                    .to_str()
                    .and_then(|s| Some(s.to_owned()))
                else {
                    return;
                };

                if let Err(e) = tx.send(Event {
                    ops: PodOps::Apply,
                    yaml,
                }) {
                    debug!("send event failed: {e}")
                };
            }
            notify::EventKind::Remove(RemoveKind::Any) => {
                let src_file = &event.paths[0];
                let Some(file_name) = src_file.file_name() else {
                    return;
                };

                let Some(parent) = src_file.parent().and_then(|d| d.file_name()) else {
                    return;
                };

                if parent != POD_APPLY_DIR {
                    return;
                }
                let Some(yaml) = root
                    .join(POD_DOWN_DIR)
                    .join(file_name)
                    .to_str()
                    .and_then(|s| Some(s.to_owned()))
                else {
                    return;
                };

                if let Err(e) = tx.send(Event {
                    ops: PodOps::Down,
                    yaml,
                }) {
                    debug!("send event failed: {e}")
                };
            }
            _ => {}
        },
        Err(e) => println!("watch error: {:?}", e),
    };

    let mut watcher = {
        let config = notify::Config::default().with_manual_polling();
        notify::PollWatcher::new(event_handler, config)?
    };
    watcher.watch(&pod_root, RecursiveMode::Recursive)?;

    while running.load(Ordering::SeqCst) {
        if let Err(e) = watcher.poll() {
            debug!("{e}");
        }
        sleep(Duration::new(1, 0));
    }

    Ok(())
}

#[cfg(not(feature = "poll_watcher"))]
pub fn watcher_loop(pod_root: PathBuf, tx: mpsc::Sender<Event>) -> anyhow::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    let event_handler = move |res: Result<notify::Event, notify::Error>| match res {
        Ok(event) => match event.kind {
            notify::EventKind::Create(CreateKind::File) => {
                let Some(yaml) = PathBuf::from(&event.paths[0])
                    .to_str()
                    .and_then(|s| Some(s.to_owned()))
                else {
                    return;
                };

                if let Err(e) = tx.send(Event {
                    ops: PodOps::Apply,
                    yaml,
                }) {
                    debug!("send event failed: {e}")
                };
            }
            notify::EventKind::Modify(kind) => {
                if let ModifyKind::Name(RenameMode::Both) = kind {
                    let src_file = &event.paths[0];
                    let des_file = &event.paths[1];

                    let Some(src_parent) = src_file.parent().and_then(|d| d.file_name()) else {
                        return;
                    };

                    let Some(des_parent) = des_file.parent().and_then(|d| d.file_name()) else {
                        return;
                    };

                    if !(src_parent == POD_APPLY_DIR && des_parent == POD_DOWN_DIR) {
                        return;
                    }

                    let Some(yaml) = des_file.to_str().and_then(|s| Some(s.to_owned())) else {
                        return;
                    };

                    if let Err(e) = tx.send(Event {
                        ops: PodOps::Down,
                        yaml,
                    }) {
                        debug!("send event failed: {e}")
                    };
                }
            }
            _ => {}
        },
        Err(e) => println!("watch error: {:?}", e),
    };
    let mut watcher = notify::recommended_watcher(event_handler)?;
    watcher.watch(&pod_root, RecursiveMode::Recursive)?;
    while running.load(Ordering::SeqCst) {
        sleep(Duration::new(1, 0));
    }

    Ok(())
}

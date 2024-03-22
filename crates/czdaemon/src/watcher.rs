use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};

use anyhow::Ok;
use log::{debug, info};
use notify::{event::CreateKind, RecursiveMode, Watcher};

use crate::worker::{Event, PodOps};
use libcz::{POD_APPLY_DIR, POD_DOWN_DIR};
#[cfg(not(feature = "poll_watcher"))]
use notify::event::{ModifyKind, RenameMode};
use std::sync::mpsc;

#[cfg(feature = "poll_watcher")]
use notify::event::RemoveKind;

fn init_signal(flag: Arc<AtomicBool>) -> anyhow::Result<()> {
    signal_hook::flag::register(signal_hook::consts::SIGINT, flag.clone())?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, flag.clone())?;
    Ok(())
}

#[cfg(feature = "poll_watcher")]
pub fn watcher_loop(pod_root: PathBuf, tx: mpsc::Sender<Event>) -> anyhow::Result<()> {
    let want_to_stop = Arc::new(AtomicBool::new(false));
    init_signal(want_to_stop.clone())?;

    let root = pod_root.clone();
    let event_handler = move |res: Result<notify::Event, notify::Error>| match res {
        Result::Ok(event) => match event.kind {
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

    while !want_to_stop.load(Ordering::SeqCst) {
        if let Err(e) = watcher.poll() {
            debug!("{e}");
        }
        sleep(Duration::new(1, 0));
    }

    info!("watcher loop exit");
    Ok(())
}

#[cfg(not(feature = "poll_watcher"))]
pub fn watcher_loop(pod_root: PathBuf, tx: mpsc::Sender<Event>) -> anyhow::Result<()> {
    let want_to_stop = Arc::new(AtomicBool::new(false));
    init_signal(want_to_stop.clone())?;

    let event_handler = move |res: Result<notify::Event, notify::Error>| match res {
        Result::Ok(event) => {
            debug!("{:?}", event);
            match event.kind {
                notify::EventKind::Create(CreateKind::File) => {
                    let path = &event.paths[0];

                    let Some(parent) = path.parent().and_then(|d| d.file_name()) else {
                        return;
                    };

                    if parent != POD_APPLY_DIR {
                        return;
                    }

                    let Some(yaml) = path.to_str().and_then(|s| Some(s.to_owned())) else {
                        return;
                    };

                    if let Err(e) = tx.send(Event {
                        ops: PodOps::Apply,
                        yaml,
                    }) {
                        debug!("send event failed: {e}")
                    };
                }
                notify::EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
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
                _ => {}
            }
        }
        Err(e) => println!("watch error: {:?}", e),
    };
    let mut watcher = notify::recommended_watcher(event_handler)?;
    watcher.watch(&pod_root, RecursiveMode::Recursive)?;
    while !want_to_stop.load(Ordering::SeqCst) {
        sleep(Duration::new(1, 0));
    }
    info!("watcher loop exit");
    Ok(())
}

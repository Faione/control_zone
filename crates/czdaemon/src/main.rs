use std::{fs, path::PathBuf, str::FromStr, sync::mpsc};

use anyhow::bail;
use clap::Parser;

use libcz::{State, INFO_DIR, IP_FILE, POD_CRUNTIME_LOG, POD_DIR, STATE_FILE};
use log::{info, warn};
use watcher::watcher_loop;
use worker::Worker;

use crate::guest::fetch_info;

mod guest;
mod watcher;
mod worker;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Opts {
    /// Sharefolder
    #[arg(short, long, required = true)]
    dir: PathBuf,

    /// Container Runtime
    #[arg(short, long, default_value = "podman")]
    cruntime: String,
}

fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    let opts = Opts::parse();
    let share_root = opts.dir.clone();

    // sync info
    if let Ok(info) = fetch_info() {
        fs::write(share_root.join(INFO_DIR).join(IP_FILE), info.ip)?;
    } else {
        warn!("fetch info failed")
    }

    // init pod watcher
    let pod_root = share_root.join(POD_DIR);
    let Some(log_file) = pod_root
        .join(POD_CRUNTIME_LOG)
        .to_str()
        .and_then(|s| Some(s.to_owned()))
    else {
        bail!("gen log file failed")
    };
    let (tx, rx) = mpsc::channel();
    let worker = Worker {
        cruntime: opts.cruntime,
        log_file,
    };
    worker.run(rx);
    info!("worker initialized");

    // sync state
    let state_file = share_root.join(INFO_DIR).join(STATE_FILE);
    let state = State::from_str(&fs::read_to_string(&state_file)?)?;
    if !state.check_update(State::Running)? {
        fs::write(state_file, State::Running.to_string())?;
    };

    watcher_loop(pod_root, tx)
}

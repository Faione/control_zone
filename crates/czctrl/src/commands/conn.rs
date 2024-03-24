use std::{fs, path::PathBuf, process::Command};

use anyhow::{anyhow, bail, Ok, Result};
use clap::Parser;
use libcz::{
    default_workdir, state::State, ControlZone, CZ_CONFIG, CZ_PRIO_KEY, INFO_DIR, IP_FILE,
    WORKDIR_ROOT,
};
use log::debug;

use crate::GloablOpts;

#[derive(Parser, Debug)]
pub struct Conn {
    /// Control Zone Config
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Name of Control Zone
    control_zone: String,
}

pub fn conn(args: Conn, global_opts: &GloablOpts) -> Result<()> {
    let full_config = match args.config {
        Some(path) => path,
        None => default_workdir(&args.control_zone).join(CZ_CONFIG),
    };

    let cz = ControlZone::new_from_full_config(&full_config)
        .map_err(|e| anyhow!("error parsing config {:#?}: {}", full_config, e))?;

    if cz.state != State::Running {
        bail!("control zone need to start fisrt")
    }

    if global_opts.dry_run {
        return Ok(());
    }
    let ip = fs::read_to_string(
        PathBuf::from(&cz.meta.share_folder)
            .join(INFO_DIR)
            .join(IP_FILE),
    )?;

    let mut cmd = Command::new("ssh");

    cmd.args(["-i", &format!("{}/{}", WORKDIR_ROOT, CZ_PRIO_KEY)])
        .args(["-o", "StrictHostKeyChecking=no"])
        .arg(&format!("root@{}", &ip));

    debug!("{:?}", cmd);
    let mut childp = match cmd.spawn() {
        Result::Ok(childp) => childp,
        Err(e) => bail!("command spawn failed: {e}"),
    };

    match childp.wait() {
        Result::Ok(code) => {
            if !code.success() {
                bail!("command exec failed: {code}")
            }
        }
        Err(e) => bail!("could not wait for command: {e}"),
    };
    Ok(())
}

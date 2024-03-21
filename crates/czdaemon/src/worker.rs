use std::{
    fs::{self, OpenOptions},
    process::{Command, Stdio},
    sync::mpsc::Receiver,
    thread,
};

use log::{debug, error, info};

#[derive(Debug)]
pub enum PodOps {
    Apply,
    Down,
}

#[derive(Debug)]
pub struct Event {
    pub ops: PodOps,
    pub yaml: String,
}

unsafe impl Sync for Event {}
unsafe impl Send for Event {}

pub struct Worker {
    pub cruntime: String,
    pub log_file: String,
}

impl Worker {
    pub fn run(&self, rx: Receiver<Event>) {
        let cruntime = self.cruntime.clone();
        let log_file = self.log_file.clone();

        thread::spawn(move || loop {
            match rx.recv() {
                Ok(event) => {
                    Command::new(&cruntime).arg("kube").arg("play");

                    let mut cmd = Command::new(&cruntime);
                    cmd.arg("kube");
                    match event.ops {
                        PodOps::Apply => {
                            info!("run pod: {}", event.yaml);
                            cmd.arg("play");
                        }
                        PodOps::Down => {
                            info!("remove pod: {}", event.yaml);
                            cmd.arg("down");
                        }
                    };

                    cmd.arg(&event.yaml);
                    debug!("{:?}", cmd);

                    let Ok(file) = OpenOptions::new().create(true).append(true).open(&log_file)
                    else {
                        error!("command run failed");
                        continue;
                    };

                    cmd.stdout(Stdio::from(file));
                    let Ok(mut cmd_fd) = cmd.spawn().map_err(|e| error!("command run failed: {e}"))
                    else {
                        continue;
                    };

                    if let Err(e) = cmd_fd.wait() {
                        error!("command run failed: {e}");
                        continue;
                    }

                    // remove downed pod yaml
                    if let Err(e) = match event.ops {
                        PodOps::Apply => Ok(()),
                        PodOps::Down => fs::remove_file(&event.yaml),
                    } {
                        error!("remove pod yaml: {} failed: {e}", event.yaml);
                        continue;
                    }

                    info!("{:?} successfully", event.ops);
                }
                Err(_) => break,
            }
        });
    }
}

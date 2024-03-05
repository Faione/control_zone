use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    str::FromStr,
};

use anyhow::{anyhow, bail, Ok, Result};
use clap::{Parser, ValueEnum};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ValueEnum)]
pub enum Monitor {
    Resctrl,
    Ebpf,
}

#[derive(Parser, Debug)]

pub struct Observe {
    /// Print Monitor Config Only
    #[arg(short, long)]
    dry_run: bool,

    /// Observe all Control Zones
    #[arg(short, long)]
    all: bool,

    /// Monitors to enable, not setting will enable all
    #[arg(short, long, value_enum, action = clap::ArgAction::Append)]
    monitor: Option<Vec<Monitor>>,

    /// Path to save obeserv conifg
    #[arg(short, long, default_value = "vm_infos.yaml")]
    output: PathBuf,

    /// Control Zones
    control_zones: Vec<String>,
}

pub fn observe(args: Observe) -> Result<()> {
    if !libutil::kvm::check_kvm() {
        bail!("kvm not enabled or not a root user")
    }

    let vm_monitor_infos: Vec<VmMonitorInfo> = if args.all || args.control_zones.len() == 0 {
        // kvm info -> proc info -> libvirt info
        libutil::kvm::get_kvm_infos()?
            .into_iter()
            .filter_map(|kvm_info| {
                let Some(tasks) = libutil::process::tasks_of(kvm_info.pid)
                    .map_err(|e| error!("parse tasks err: {}", e))
                    .ok()
                else {
                    return None;
                };

                let Some(libvirt_info) = libutil::process::libvirt_info_of(kvm_info.pid)
                    .map_err(|e| error!("parse libvirt info err: {}", e))
                    .ok()
                else {
                    return None;
                };

                Some(VmMonitorInfo {
                    pid: kvm_info.pid,
                    kvm_debug_dir: kvm_info.kvm_debug_dir,
                    tasks: tasks,
                    vm_id: libvirt_info.vm_id,
                    vm_cgroup: libvirt_info.vm_cgroup,
                    vm_name: libvirt_info.vm_name,
                })
            })
            .collect()
    } else {
        todo!()
    };

    debug!("{:#?}", vm_monitor_infos);
    let vm_monitor_config = serde_yaml::to_string(&vm_monitor_infos)?;

    // just print monitor config
    if args.dry_run {
        println!("{}", vm_monitor_config);
        return Ok(());
    }

    let monitor_set = match args.monitor {
        Some(monitors) => HashSet::from_iter(monitors.into_iter()),
        None => HashSet::from([Monitor::Resctrl, Monitor::Ebpf]),
    };

    vm_monitor_infos.into_iter().for_each(|vm_monitor_info| {
        monitor_set.iter().for_each(|monitor| match monitor {
            Monitor::Resctrl => {
                if let Err(e) = vm_monitor_info.init_resctrl_mgroup() {
                    error!(
                        "init resctrl mon group for {} failed: {}",
                        vm_monitor_info.vm_name, e
                    );
                } else {
                    info!(
                        "resctrl mon group for {} initialized",
                        vm_monitor_info.vm_name
                    );
                }
            }
            Monitor::Ebpf => {
                if let Err(e) = vm_monitor_info.init_ebpf_cgroup() {
                    error!(
                        "init ebpf cgroup for {} failed: {}",
                        vm_monitor_info.vm_name, e
                    );
                } else {
                    info!("ebpf cgroup for {} initialized", vm_monitor_info.vm_name);
                }
            }
        })
    });

    let mut config_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(args.output)?;

    write!(config_file, "{}", vm_monitor_config)?;

    Ok(())
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct VmMonitorInfo {
    pub pid: u32,
    pub kvm_debug_dir: String,
    pub tasks: Vec<u32>,
    pub vm_id: u32,
    pub vm_name: String,
    pub vm_cgroup: String,
}

static RESCTL_ROOT: &str = "/sys/fs/resctrl";

impl VmMonitorInfo {
    /// init resctrl monitor group for Virtual Machine
    fn init_resctrl_mgroup(&self) -> Result<()> {
        let resctl_root = PathBuf::from_str(RESCTL_ROOT)?;

        if !resctl_root.exists() && !resctl_root.is_dir() {
            return Err(anyhow!("resctrl not enabled"));
        }

        let mgroup_dir = resctl_root.join(PathBuf::from(format!("mon_groups/{}", self.vm_name)));

        // clean old tasks
        if mgroup_dir.exists() {
            fs::remove_dir(&mgroup_dir)?;
        }
        fs::create_dir(&mgroup_dir)?;
        debug!("resctrl mon group created for {}", self.vm_name);

        let mut file = OpenOptions::new()
            .write(true)
            .open(mgroup_dir.join(PathBuf::from("tasks")))?;

        for task in &self.tasks {
            if let Err(e) = write!(file, "{}", task) {
                error!("write {} to tasks failed: {}", task, e);
            }
        }

        debug!("{} tasks added for resctrl mon group", self.tasks.len(),);
        Ok(())
    }

    fn init_ebpf_cgroup(&self) -> Result<()> {
        todo!()
    }
}

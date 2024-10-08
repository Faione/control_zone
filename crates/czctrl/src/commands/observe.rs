use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    str::FromStr,
};

use anyhow::{anyhow, bail, Ok, Result};
use clap::{Parser, ValueEnum};
use libbpfmap::CgroupMapWrapper;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

use crate::{
    config::{CGROUP_ROOT, RESCTL_ROOT},
    GloablOpts,
};

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ValueEnum)]
pub enum Monitor {
    Resctrl,
    Ebpf,
}

#[derive(Parser, Debug)]
pub struct Observe {
    /// Observe all Control Zones
    #[arg(short, long)]
    all: bool,

    /// Clean exsit Monitors
    #[arg(short, long)]
    clean: bool,

    /// Monitors to enable, not setting will enable all
    #[arg(short, long, value_enum, action = clap::ArgAction::Append)]
    monitor: Option<Vec<Monitor>>,

    /// Path to save obeserv conifg
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Control Zones
    control_zones: Vec<String>,
}

#[derive(Debug)]
enum Action {
    Show,
    Init,
    Clean,
}

pub fn observe(args: Observe, global_opts: &GloablOpts) -> Result<()> {
    if !libutil::kvm::check_kvm() {
        bail!("kvm not enabled or not a root user")
    }

    // kvm info -> proc info -> libvirt info
    let mut vm_monitor_infos: Vec<VmMonitorInfo> = libutil::kvm::get_kvm_infos()?
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
        .collect();

    if !args.all && args.control_zones.len() != 0 {
        let mut control_zones: HashSet<String> = HashSet::from_iter(args.control_zones.into_iter());
        vm_monitor_infos = vm_monitor_infos
            .into_iter()
            .filter(|vm_monitor_info| {
                let pre_len = control_zones.len();
                control_zones.retain(|e| !vm_monitor_info.same_as(e));

                // if vm_monitor_info in args HashSet
                // then it will be moved from HashSet
                // and we will keep it in new vm_monitor_infos
                pre_len != control_zones.len()
            })
            .collect();
    }

    let monitor_set = match args.monitor {
        Some(monitors) => HashSet::from_iter(monitors.into_iter()),
        None => HashSet::from([Monitor::Resctrl, Monitor::Ebpf]),
    };

    let action = if global_opts.dry_run {
        Action::Show
    } else if args.clean {
        Action::Clean
    } else {
        Action::Init
    };

    debug!("{:#?}", vm_monitor_infos);
    let vm_monitor_config = serde_yaml::to_string(&vm_monitor_infos)?;
    match action {
        Action::Show => {
            println!("--------VM Monitor Config--------\n");
            println!("{}", vm_monitor_config);

            monitor_set.iter().for_each(|monitor| match monitor {
                Monitor::Resctrl => {
                    println!("\n--------Exist Resctrl Monitor Groups--------\n");
                    let resctl_mon_group_root = PathBuf::from(RESCTL_ROOT).join("mon_groups");
                    if !resctl_mon_group_root.exists() && !resctl_mon_group_root.is_dir() {
                        error!("resctrl subsystem maybe not enabled");
                    }

                    let Some(readir) = fs::read_dir(resctl_mon_group_root)
                        .map_err(|e| error!("read mon groups failed {e}"))
                        .ok()
                    else {
                        return;
                    };

                    let mon_groups: Vec<String> = readir
                        .filter_map(|entry| entry.ok())
                        .filter(|entry| entry.path().is_dir())
                        .filter_map(|dir| {
                            dir.path()
                                .to_str()
                                .and_then(|dir_str| Some(dir_str.to_owned()))
                        })
                        .collect();

                    println!("{mon_groups:#?}")
                }
                Monitor::Ebpf => {
                    let Some(wrapper) = libbpfmap::CgroupMapWrapper::new()
                        .map_err(|e| error!("init cgroup map error: {}", e))
                        .ok()
                    else {
                        return;
                    };

                    println!("\n--------Exist Cgroup Ebpf Map--------\n");
                    wrapper.list();
                }
            });
            Ok(())
        }
        Action::Init => {
            monitor_set.iter().for_each(|monitor| match monitor {
                Monitor::Resctrl => vm_monitor_infos.iter().for_each(|vm_monitor_info| {
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
                }),
                Monitor::Ebpf => {
                    let Some(wrapper) = libbpfmap::CgroupMapWrapper::new()
                        .map_err(|e| error!("init cgroup map error: {}", e))
                        .ok()
                    else {
                        return;
                    };

                    vm_monitor_infos.iter().for_each(|vm_monitor_info| {
                        if let Err(e) = vm_monitor_info.init_ebpf_cgroup(&wrapper) {
                            error!(
                                "init ebpf cgroup for {} failed: {}",
                                vm_monitor_info.vm_name, e
                            );
                        } else {
                            info!("ebpf cgroup for {} initialized", vm_monitor_info.vm_name);
                        }
                    });
                }
            });

            let Some(output) = args.output else {
                return Ok(());
            };

            let mut config_file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&output)?;

            write!(config_file, "{vm_monitor_config}")?;
            info!("monior conifg saved at {:#?}", output);
            Ok(())
        }
        Action::Clean => {
            monitor_set.iter().for_each(|monitor| match monitor {
                Monitor::Resctrl => vm_monitor_infos.iter().for_each(|vm_monitor_info| {
                    if let Err(e) = vm_monitor_info.remove_resctrl_group() {
                        error!(
                            "init resctrl mon group for {} failed: {}",
                            vm_monitor_info.vm_name, e
                        );
                    } else {
                        info!("resctrl mon group for {} cleaned", vm_monitor_info.vm_name);
                    }
                }),
                Monitor::Ebpf => {
                    let Some(wrapper) = libbpfmap::CgroupMapWrapper::new()
                        .map_err(|e| error!("init cgroup map error: {}", e))
                        .ok()
                    else {
                        return;
                    };

                    vm_monitor_infos.iter().for_each(|vm_monitor_info| {
                        if let Err(e) = vm_monitor_info.remove_ebpf_cgroup(&wrapper) {
                            error!(
                                "init ebpf cgroup for {} failed: {}",
                                vm_monitor_info.vm_name, e
                            );
                        } else {
                            info!("ebpf cgroup for {} cleaned", vm_monitor_info.vm_name);
                        }
                    });
                }
            });

            let Some(output) = args.output else {
                return Ok(());
            };

            let mut config_file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&output)?;

            write!(config_file, "")?;
            info!("monior conifg cleaned at {:#?}", &output);
            Ok(())
        }
    }
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

impl VmMonitorInfo {
    fn same_as(&self, s: &str) -> bool {
        match s.parse::<u32>() {
            Result::Ok(vm_id) => self.vm_id == vm_id,
            Err(_) => self.vm_name == s,
        }
    }
}

/// Resctrl Option
impl VmMonitorInfo {
    /// init resctrl monitor group for Virtual Machine
    fn init_resctrl_mgroup(&self) -> Result<()> {
        let mgroup_dir = self.remove_resctrl_group()?;
        fs::create_dir(&mgroup_dir)?;
        debug!("resctrl mon group created for {}", self.vm_name);

        let mut file = OpenOptions::new()
            .write(true)
            .open(mgroup_dir.join(PathBuf::from("tasks")))?;

        let mut task_count = 0;
        for task in &self.tasks {
            if let Err(e) = write!(file, "{}", task) {
                error!("write {task} to tasks failed: {e}");
            } else {
                task_count += 1;
            }
        }

        debug!(
            "{} tasks added for resctrl mon group, {} added failed",
            task_count,
            self.tasks.len() - task_count
        );
        Ok(())
    }

    fn remove_resctrl_group(&self) -> Result<PathBuf> {
        let resctl_root = PathBuf::from_str(RESCTL_ROOT)?;

        if !resctl_root.exists() && !resctl_root.is_dir() {
            return Err(anyhow!("resctrl not enabled"));
        }

        let mgroup_dir = resctl_root.join(PathBuf::from(format!("mon_groups/{}", self.vm_name)));

        if mgroup_dir.exists() {
            fs::remove_dir(&mgroup_dir)?;
        }
        Ok(mgroup_dir)
    }
}

/// Ebpf Cgroup Option
impl VmMonitorInfo {
    fn detect_libvirt_cgroup(&self) -> Result<Vec<String>> {
        let cgroup_root_dir = PathBuf::from(format!("{CGROUP_ROOT}{}", &self.vm_cgroup));
        if !cgroup_root_dir.exists() || !cgroup_root_dir.is_dir() {
            bail!("{} is not a cgroup dir", self.vm_cgroup);
        }

        Ok(fs::read_dir(cgroup_root_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|dir| {
                dir.path()
                    .to_str()
                    .and_then(|dir_str| Some(dir_str.to_owned()))
            })
            .collect())
    }

    fn init_ebpf_cgroup(&self, bpf_map: &CgroupMapWrapper) -> Result<()> {
        bpf_map.insert_list(&self.detect_libvirt_cgroup()?)
    }

    fn remove_ebpf_cgroup(&self, bpf_map: &CgroupMapWrapper) -> Result<()> {
        bpf_map.delete_list(&self.detect_libvirt_cgroup()?)
    }
}

use anyhow::{Ok, Result};
use regex::Regex;
use std::{fs, path::Path};

static KVM_DEBUG_FS: &str = "/sys/kernel/debug/kvm";
static RE_KVM_DIR_PID: &str = r"^(\d+)-\d+$";

#[derive(Debug, Clone)]
pub struct KVMInfo {
    pub pid: u32,
    pub kvm_debug_dir: String,
}

/// check if kvm enabled and debugfs exists
pub fn check_kvm() -> bool {
    // check is root
    if unsafe { libc::getuid() != 0 } {
        return false;
    }

    let path = Path::new(KVM_DEBUG_FS);
    return path.exists() && path.is_dir();
}

/// read all vm kvm info from kvm debugfs
pub fn get_kvm_infos() -> Result<Vec<KVMInfo>> {
    let root_path = Path::new(KVM_DEBUG_FS);
    let re = Regex::new(RE_KVM_DIR_PID)?;

    let kvm_infos = fs::read_dir(root_path)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                path.file_name().and_then(|n| n.to_str()).and_then(|fname| {
                    re.captures(fname)
                        .and_then(|caps| caps.get(1))
                        .and_then(|matched| matched.as_str().parse::<u32>().ok())
                        .and_then(|pid| {
                            Some(KVMInfo {
                                pid: pid,
                                kvm_debug_dir: fname.to_string(),
                            })
                        })
                })
            } else {
                None
            }
        })
        .collect();

    Ok(kvm_infos)
}

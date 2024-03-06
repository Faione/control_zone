use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Ok, Result};
use regex::Regex;

const PROC_FS: &str = "/proc";
const RE_PROC_LIBVIRT_CMDLINE: &str = r"/domain-(?<vm_id>\d+)-(?<vm_name>.+)/master-key.aes";

/// List tasks of Process pid
pub fn tasks_of(pid: u32) -> Result<Vec<u32>> {
    let proc_dir = Path::new(PROC_FS);
    let task_dir = proc_dir.join(PathBuf::from(format!("{pid}/task")));

    if !task_dir.exists() || !task_dir.is_dir() {
        bail!("process {pid} not exist")
    }

    let tasks = fs::read_dir(task_dir)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .and_then(|fname| fname.parse::<u32>().ok())
        })
        .collect();

    Ok(tasks)
}

pub struct LibvirtInfo {
    pub vm_id: u32,
    pub vm_name: String,
    pub vm_cgroup: String,
}

/// Read libvirt info of a libvirt process
pub fn libvirt_info_of(pid: u32) -> Result<LibvirtInfo> {
    let proc_root = Path::new(PROC_FS).join(PathBuf::from(pid.to_string()));

    if !proc_root.exists() || !proc_root.is_dir() {
        bail!("process {} not exist", pid)
    }

    let cmdline = proc_root.join(PathBuf::from("cmdline"));
    if !cmdline.exists() || !cmdline.is_file() {
        bail!("process {pid} don't have a libvirt cmdline on {cmdline:#?}",)
    }

    let cgroup = proc_root.join(PathBuf::from("cgroup"));
    if !cgroup.exists() || !cgroup.is_file() {
        bail!("process {pid} don't have a libvirt cgroup on {cgroup:#?}",)
    }

    let cmdline_str = fs::read_to_string(cmdline)?;
    let cgroup_str = fs::read_to_string(cgroup)?;

    let (vm_id, vm_name) = parse_libvirt_cmdline(&cmdline_str)?;
    let vm_cgroup = parse_libvirt_cgroup(&cgroup_str)?;

    Ok(LibvirtInfo {
        vm_id,
        vm_name,
        vm_cgroup,
    })
}

fn parse_libvirt_cmdline(cmdline: &str) -> Result<(u32, String)> {
    let re = Regex::new(RE_PROC_LIBVIRT_CMDLINE)?;

    let Some(caps) = re.captures(cmdline) else {
        bail!("not a valid libvirt cmdline")
    };

    let vm_id: u32 = (&caps["vm_id"]).parse::<u32>()?;
    let vm_name = (&caps["vm_name"]).to_owned();

    Ok((vm_id, vm_name))
}

#[inline]
fn parse_libvirt_cgroup(cgroup: &str) -> Result<String> {
    Ok(cgroup
        .trim_start_matches("0::")
        .trim_end()
        .trim_end_matches("/emulator")
        .to_owned())
}

#[cfg(test)]

mod test {
    use super::parse_libvirt_cgroup;

    #[test]
    fn moke_parse_libvirt_cgroup() {
        let cgroup =
            "0::/machine.slice/machine-qemu\\x2d32\\x2dcontrolzonedefault.scope/libvirt/emulator\n";

        let cg = parse_libvirt_cgroup(cgroup).unwrap();
        assert_eq!(
            cg,
            r"/machine.slice/machine-qemu\x2d32\x2dcontrolzonedefault.scope/libvirt"
        )
    }
}

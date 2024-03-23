use std::{fs, path::PathBuf, process::Command};

use anyhow::{bail, Ok};
use libcz::vruntime::VRuntime;
use log::debug;

const QEMU_BIN: &str = "qemu-system-x86_64";
const QEMU_KILLER: &str = "kill";
const QEMU_PID_FILE: &str = "qpid";

pub struct Qemu {}

#[inline]
fn pid_file(workdir: &str) -> Option<String> {
    PathBuf::from(workdir)
        .join(QEMU_PID_FILE)
        .to_str()
        .and_then(|os_str| Some(os_str.to_owned()))
}

impl VRuntime for Qemu {
    fn start(&self, cz: &mut libcz::ControlZone) -> anyhow::Result<()> {
        let Some(pid_file) = pid_file(&cz.meta.workdir) else {
            bail!("error gen qemu pid file")
        };

        let mut cmd = Command::new(QEMU_BIN);
        cmd.args([
            "-enable-kvm",
            "-display",
            "none",
            "-daemonize",
            "-machine",
            "pc",
            "-cpu",
            "host",
        ]);
        cmd.args(["-pidfile", &pid_file]);

        // Resource
        cmd.args(["-smp", &format!("{}", cz.resource.cpus.len())]);
        cmd.args(["-m", &format!("{}", cz.resource.memory)]);

        if cz.resource.static_net.is_none() {
            bail!("qemu vruntime currently not support dynamic IP")
        }
        cmd.args([
            "-device",
            "virtio-net-pci,netdev=net",
            "-netdev",
            "bridge,br=br0,id=net",
        ]);

        // Meta
        cmd.args(["-name", &cz.meta.name]);
        //   ShareFolder
        cmd.args([
            "-device",
            "virtio-9p-pci,fsdev=shared-folder,mount_tag=hostshare",
        ]);
        cmd.args([
            "-fsdev",
            &format!(
                "local,id=shared-folder,path={},security_model=mapped",
                cz.meta.share_folder
            ),
        ]);

        // OS
        //   Rootfs
        cmd.args(["-device", "virtio-blk-pci,drive=hd"]);
        cmd.args(["-drive", &format!("file={},if=none,id=hd", cz.os.rootfs)]);
        cmd.args(["-kernel", &cz.os.kernel]);

        //  TODO: initramfs
        cmd.args(["-append", &cz.os.kcmdline]);

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

    // stop qmeu vm
    fn stop(&self, cz: &mut libcz::ControlZone) -> anyhow::Result<()> {
        let Some(pid_file) = pid_file(&cz.meta.workdir) else {
            bail!("error gen qemu pid file")
        };
        let pid_s = fs::read_to_string(&pid_file)?;

        // TODO: Stop More gently
        let mut cmd = Command::new(QEMU_KILLER);
        cmd.arg(&pid_s.trim());

        let mut childp = match cmd.spawn() {
            Result::Ok(childp) => childp,
            Err(e) => bail!("command spawn failed: {e}"),
        };
        debug!("{:?}", cmd);
        match childp.wait() {
            Result::Ok(code) => {
                if !code.success() {
                    bail!("command exec failed: {code}")
                }
            }
            Err(e) => bail!("could not wait for command: {e}"),
        };

        // remove old pid file
        fs::remove_file(pid_file)?;
        Ok(())
    }
}

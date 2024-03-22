use anyhow::{anyhow, bail, Ok};
use log::debug;
use serde::{Deserialize, Serialize};
use std::{fmt::Write, fs, path::PathBuf, str::FromStr};

use self::{
    czos::CZOS,
    meta::{Meta, MetaBuilder},
    resource::Resource,
};

pub use self::state::State;

mod czos;
mod meta;
mod resource;
mod state;
mod util;

#[cfg(test)]
mod test;

// ControlZone
pub const WORKDIR_ROOT: &str = "/tmp/controlzones";
pub const CZ_CONFIG: &str = "controlzone.yaml";
pub const CZ_IMAGE: &str = "cz.img";

pub const POD_DIR: &str = "pod";
// pod/apply
pub const POD_APPLY_DIR: &str = "apply";
// pod/apply
pub const POD_DOWN_DIR: &str = "down";
// pod/log
pub const POD_CRUNTIME_LOG: &str = "log";

// sharefolder/info/
pub const INFO_DIR: &str = "info";
// sharefolder/info/state
pub const STATE_FILE: &str = "state";
// sharefolder/info/ip
pub const IP_FILE: &str = "ip";
// sharefolder/info/static_net
pub const STATIC_NET_FILE: &str = "static_net";

#[inline]
pub fn default_workdir(cz_name: &str) -> PathBuf {
    PathBuf::from(WORKDIR_ROOT).join(cz_name)
}

#[derive(Debug)]
pub enum UpdateMode {
    // Os changed
    Reboot,
    // Resource Changed but Os not changed
    Hot,
    // Nothing changed
    Stale,
}

/// check state update and debug wrap
macro_rules! check_update {
    ($old_state:expr, $new_state:expr) => {
        if $old_state.check_update($new_state)? {
            debug!("control zone keep stale: {}", $old_state);
            return Ok(());
        }
    };
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlZone {
    #[serde(default = "Meta::default")]
    pub meta: Meta,
    pub os: CZOS,
    pub resource: Resource,

    #[serde(skip)]
    pub state: State,
}

impl ControlZone {
    pub fn new_from_full_config(file: &PathBuf) -> anyhow::Result<Self> {
        let config = fs::read_to_string(file)?;

        let mut cz: ControlZone = serde_yaml::from_str(&config)?;
        if !cz.meta.is_valid() {
            bail!("not a created control zone")
        }

        let state_file = cz.state_file();
        cz.state = if !state_file.exists() {
            bail!("not a created control zone")
        } else {
            State::from_str(&fs::read_to_string(state_file)?)?
        };
        cz.resource.gen_cpus();
        Ok(cz)
    }

    pub fn new_from_config(file: &PathBuf) -> anyhow::Result<Self> {
        let config = fs::read_to_string(file)?;
        let mut cz: ControlZone = serde_yaml::from_str(&config)?;

        // init meta
        cz.meta = MetaBuilder::new(cz.meta, file)?
            .with_share_folder()?
            .with_full_config()?
            .build()?;

        // init resource
        cz.resource.gen_cpus();

        // init state
        let state_file = cz.state_file();
        cz.state = if !state_file.exists() {
            State::Pending
        } else {
            State::from_str(&fs::read_to_string(state_file)?)?
        };

        Ok(cz)
    }

    #[inline]
    fn state_file(&self) -> PathBuf {
        PathBuf::from(&self.meta.share_folder)
            .join(INFO_DIR)
            .join(STATE_FILE)
    }

    /// to libvirt virtual machine xml config
    pub fn to_xml(&self) -> anyhow::Result<String> {
        let mut buf = String::from("<domain type='kvm'>\n");

        // Init name
        writeln!(&mut buf, "<name>{}</name>", self.meta.name)?;

        // Init memory
        writeln!(
            &mut buf,
            "<memory unit='MB'>{}</memory>",
            self.resource.memory
        )?;

        // Init static CPU
        writeln!(
            &mut buf,
            "<vcpu placement='static'>{}</vcpu>",
            self.resource.cpus.len()
        )?;
        writeln!(&mut buf, "<cputune>")?;
        for (i, cpu) in self.resource.cpus.iter().enumerate() {
            writeln!(&mut buf, "<vcpupin vcpu='{}' cpuset='{}'/>", i, cpu)?;
        }
        writeln!(&mut buf, "</cputune>")?;

        // Init Rootfs
        let rootfs = format!(
            "\
            <disk type='file' device='disk'>\n\
            <driver name='qemu' type='qcow2'/>\n\
            <source file='{}'/>\n\
            #<target dev='vda' bus='virtio'/>\n\
            <alias name='ua-box-volume-0'/>\n\
            <address type='pci' domain='0x0000' bus='0x00' slot='0x02' function='0x0'/>\n\
            </disk>",
            self.os.rootfs
        );

        // Init Network
        // if static ip configured, then only using bridge network
        let network = match &self.resource.static_net {
            Some(_) => String::from(
                "\
                <interface type='bridge'>\n\
                <source bridge='br0'/>\n\
                <model type='virtio'/>\n\
                <alias name='ua-net-1'/>\n\
                <address type='pci' domain='0x0000' bus='0x00' slot='0x05' function='0x0'/>\n\
                </interface>",
            ),
            None => format!(
                "\
                    <interface type='network'>\n\
                    <domain name='{}'/>\n\
                    <source network='default'/>\n\
                    <model type='virtio'/>\n\
                    <driver iommu='off'/>\n\
                    <alias name='ua-net-0'/>\n\
                    <address type='pci' domain='0x0000' bus='0x00' slot='0x04' function='0x0'/>\n\
                    </interface>\n\
                    <interface type='bridge'>\n\
                    <source bridge='br0'/>\n\
                    <model type='virtio'/>\n\
                    <alias name='ua-net-1'/>\n\
                    <address type='pci' domain='0x0000' bus='0x00' slot='0x05' function='0x0'/>\n\
                    </interface>",
                self.meta.name
            ),
        };

        // Init Sharefolder
        let sharefolder = format!(
            "\
            <filesystem type='mount' accessmode='passthrough'>\n\
            <driver type='virtiofs' queue='1024'/>\n\
            <source dir='{}'/>\n\
            <target dir='hostshare'/>\n\
            </filesystem>",
            self.meta.share_folder
        );

        // Init OS
        writeln!(
            &mut buf,
            "<os>\n<type arch='x86_64' machine='pc-i440fx-jammy'>hvm</type>"
        )?;
        writeln!(&mut buf, "<kernel>{}</kernel>", self.os.kernel)?;
        if let Some(initrd) = &self.os.initram_fs {
            writeln!(&mut buf, "<initrd>{}</initrd>", initrd)?;
        }
        writeln!(&mut buf, "<cmdline>{}</cmdline>", self.os.kcmdline)?;

        write!(
            &mut buf,
            "<boot dev='hd'/>
<bootmenu enable='no'/>
</os>
<features>
<acpi/>
<apic/>
<pae/>
</features>
<cpu mode='host-model' check='partial'/>
<clock offset='utc'/>
<on_poweroff>destroy</on_poweroff>
<on_reboot>restart</on_reboot>
<on_crash>destroy</on_crash>
<memoryBacking>
<source type='memfd'/>
<access mode='shared'/>
</memoryBacking>
<devices>
<emulator>/usr/bin/qemu-system-x86_64</emulator>
{}
{}
<serial type='pty'>
<target type='isa-serial' port='0'>
<model name='isa-serial'/>
</target>
</serial>
<console type='pty'>
<target type='serial' port='0'/>
</console>
<input type='mouse' bus='ps2'/>
<memballoon model='virtio'>
<address type='pci' domain='0x0000' bus='0x00' slot='0x03' function='0x0'/>
</memballoon>
{}
</devices>
</domain>",
            rootfs, network, sharefolder
        )?;
        Ok(buf)
    }
}

impl ControlZone {
    pub fn test_exists(&self) -> Option<PathBuf> {
        let workdir = PathBuf::from(&self.meta.workdir);
        if workdir.exists() {
            Some(workdir)
        } else {
            None
        }
    }

    /// delete workdir of controlzone
    pub fn delete_workdir(&self) -> anyhow::Result<()> {
        if let Some(workdir) = self.test_exists() {
            fs::remove_dir_all(&workdir)?;
        }
        Ok(())
    }

    /// init workdir for control zone
    /// copy image from src to des
    pub fn init_workdir(&mut self) -> anyhow::Result<()> {
        let workdir = PathBuf::from(&self.meta.workdir);
        fs::create_dir_all(&workdir)?;

        // copy rootfs
        let src_rootfs = PathBuf::from(&self.os.rootfs);
        let des_rootfs = workdir.join(CZ_IMAGE);
        fs::copy(src_rootfs, &des_rootfs)?;
        self.os.rootfs = des_rootfs
            .to_str()
            .ok_or(anyhow!("parse rootfs failed"))?
            .to_owned();

        // create sharefolder
        let share_folder = PathBuf::from(&self.meta.share_folder);
        fs::create_dir(&share_folder)?;

        // create pod dir
        fs::create_dir(share_folder.join(POD_DIR))?;

        // create info dir
        fs::create_dir(share_folder.join(INFO_DIR))?;
        fs::write(share_folder.join(INFO_DIR).join(IP_FILE), "Non")?;
        if let Some(static_ip) = &self.resource.static_net {
            fs::write(
                share_folder.join(INFO_DIR).join(STATIC_NET_FILE),
                static_ip.to_interface_cfg()?,
            )?;
        }

        Ok(())
    }

    pub fn sync_to_file(&self) -> anyhow::Result<()> {
        fs::write(&self.meta.full_config, serde_yaml::to_string(self)?)?;
        Ok(())
    }

    fn sync_state(&mut self, state: State) -> anyhow::Result<()> {
        fs::write(self.state_file(), state.to_string())?;
        self.state = state;
        Ok(())
    }

    pub fn update_config(&mut self, new_cz: Self) -> anyhow::Result<UpdateMode> {
        if new_cz.meta != self.meta {
            bail!("meta data must not be changed!")
        }

        let mut mode: UpdateMode = UpdateMode::Stale;
        if new_cz.resource != self.resource {
            debug!("update and keep running");
            self.resource.update(new_cz.resource)?;
            mode = UpdateMode::Hot
        }

        if new_cz.os != self.os {
            debug!("update and reboot");
            self.os.update(new_cz.os)?;
            mode = UpdateMode::Reboot
        }

        self.sync_to_file()?;
        Ok(mode)
    }
}

impl ControlZone {
    pub fn create(&mut self) -> anyhow::Result<()> {
        let state = State::Created;
        check_update!(self.state, state);

        if let Err(e) = self.init_workdir() {
            self.delete_workdir()?;
            bail!(e);
        }

        if let Err(e) = self.sync_to_file() {
            self.delete_workdir()?;
            bail!(e);
        }

        if let Err(e) = self.sync_state(state) {
            self.delete_workdir()?;
            bail!(e);
        }
        Ok(())
    }

    pub fn start<Fs, Fw>(&mut self, start_f: Fs, wait_f: Option<Fw>) -> anyhow::Result<()>
    where
        Fs: Fn(&ControlZone) -> anyhow::Result<()>,
        Fw: Fn(&PathBuf) -> anyhow::Result<State>,
    {
        let state = State::Running;
        check_update!(self.state, state);

        start_f(self)?;

        if let Some(wait_f) = wait_f {
            self.state = wait_f(&self.state_file())?;
        }
        Ok(())
    }

    pub fn stop<F>(&mut self, stop_f: F) -> anyhow::Result<()>
    where
        F: Fn(&ControlZone) -> anyhow::Result<()>,
    {
        let state = State::Stopped;
        check_update!(self.state, state);

        stop_f(&self)?;
        if let Err(e) = self.sync_state(state) {
            bail!(e);
        }
        Ok(())
    }

    pub fn remove(&mut self) -> anyhow::Result<()> {
        let state = State::Zombied;
        check_update!(self.state, state);

        if let Err(e) = self.delete_workdir() {
            bail!(e);
        }
        // do not have to save state
        self.state = state;
        Ok(())
    }
}

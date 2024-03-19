use anyhow::{anyhow, bail, Ok};
use serde::{Deserialize, Serialize};
use std::{fmt::Write, fs, path::PathBuf, str::FromStr};

use crate::config::WORKDIR_ROOT;

use self::{
    czos::CZOS,
    meta::{Meta, MetaBuilder},
    resource::Resource,
    state::State,
    util::parse_cpuset,
};

mod czos;
mod meta;
mod resource;
mod state;
mod util;

#[cfg(test)]
mod test;

#[inline]
pub fn default_workdir(cz_name: &str) -> PathBuf {
    PathBuf::from(WORKDIR_ROOT).join(PathBuf::from(cz_name))
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
        PathBuf::from(&self.meta.workdir).join(PathBuf::from("state"))
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
<devices>
<emulator>/usr/bin/qemu-system-x86_64</emulator>
<disk type='file' device='disk'>
<driver name='qemu' type='qcow2'/>
<source file='{}'/>
#<target dev='vda' bus='virtio'/>
<alias name='ua-box-volume-0'/>
<address type='pci' domain='0x0000' bus='0x00' slot='0x02' function='0x0'/>
</disk>
<interface type='network'>
<domain name='{}'/>
<source network='default'/>
<model type='virtio'/>
<driver iommu='off'/>
<alias name='ua-net-0'/>
<address type='pci' domain='0x0000' bus='0x00' slot='0x04' function='0x0'/>
</interface>
<interface type='bridge'>
<source bridge='br0'/>
<model type='virtio'/>
<alias name='ua-net-1'/>
<address type='pci' domain='0x0000' bus='0x00' slot='0x05' function='0x0'/>
</interface>
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
<filesystem type='mount' accessmode='mapped'>
<source dir='{}'/>
<target dir='hostshare'/>
<address type='pci' domain='0x0000' bus='0x00' slot='0x06' function='0x0'/>
</filesystem>
</devices>
</domain>",
            self.os.rootfs, self.meta.name, self.meta.share_folder
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
        let des_rootfs = workdir.join(PathBuf::from("cz.img"));
        fs::copy(src_rootfs, &des_rootfs)?;
        self.os.rootfs = des_rootfs
            .to_str()
            .ok_or(anyhow!("parse rootfs failed"))?
            .to_owned();

        // create sharefolder
        let share_folder = PathBuf::from(&self.meta.share_folder);
        fs::create_dir(&share_folder)?;

        Ok(())
    }

    fn sync_to_file(&self) -> anyhow::Result<()> {
        fs::write(&self.meta.full_config, serde_yaml::to_string(self)?)?;
        Ok(())
    }

    fn sync_state(&mut self, state: State) -> anyhow::Result<()> {
        fs::write(self.state_file(), state.to_string())?;
        self.state = state;
        Ok(())
    }
}

impl ControlZone {
    pub fn create(&mut self) -> anyhow::Result<()> {
        let (state, stale) = self.state.check_update(State::Created)?;
        if stale {
            return Ok(());
        }

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

    pub fn start<F>(&mut self, start_f: F) -> anyhow::Result<()>
    where
        F: Fn(&ControlZone) -> anyhow::Result<()>,
    {
        let (state, stale) = self.state.check_update(State::Running)?;
        if stale {
            return Ok(());
        }
        start_f(&self)?;
        if let Err(e) = self.sync_state(state) {
            bail!(e);
        }
        Ok(())
    }

    pub fn stop<F>(&mut self, stop_f: F) -> anyhow::Result<()>
    where
        F: Fn(&ControlZone) -> anyhow::Result<()>,
    {
        let (state, stale) = self.state.check_update(State::Stopped)?;
        if stale {
            return Ok(());
        }

        stop_f(&self)?;
        if let Err(e) = self.sync_state(state) {
            bail!(e);
        }
        Ok(())
    }

    pub fn remove(&mut self) -> anyhow::Result<()> {
        let (state, stale) = self.state.check_update(State::Zombied)?;
        if stale {
            return Ok(());
        }

        if let Err(e) = self.delete_workdir() {
            bail!(e);
        }
        self.state = state;
        Ok(())
    }
}

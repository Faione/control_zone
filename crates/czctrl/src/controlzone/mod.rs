use anyhow::{anyhow, bail, Ok};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fmt::Write, fs, path::PathBuf};

use self::{
    czos::CZOS,
    meta::{Meta, MetaBuilder},
    resource::Resource,
    state::State,
};

mod czos;
mod meta;
mod resource;
mod state;
mod util;

#[cfg(test)]
mod test;

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
    pub fn new_from_config(file: &PathBuf) -> anyhow::Result<Self> {
        let config = fs::read_to_string(file)?;
        let mut cz: ControlZone = serde_yaml::from_str(&config)?;

        // init meta
        cz.meta = MetaBuilder::new(cz.meta, file)?
            .with_share_folder()?
            .with_full_config()?
            .build()?;

        // init resource
        cz.resource.cpus = util::parse_cpuset(&cz.resource.cpuset)
            .into_iter()
            .collect();

        Ok(cz)
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
        let workdir = self.test_exists().ok_or(anyhow!("workdir exists"))?;
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

        // save full config
        fs::write(&self.meta.full_config, serde_yaml::to_string(self)?)?;

        Ok(())
    }
}

impl ControlZone {
    fn update_state(&mut self, new_state: State) -> anyhow::Result<()> {
        self.state = new_state;

        Ok(())
    }
    pub fn create(mut self) -> anyhow::Result<Self> {
        match self.state {
            State::Pendding => {
                self.init_workdir()?;
            }
            _ => bail!("invalied option"),
        }

        todo!()
    }
}

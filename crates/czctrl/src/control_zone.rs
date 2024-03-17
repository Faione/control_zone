use anyhow::{anyhow, Ok};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fmt::Write, fs, path::PathBuf};

const WORKDIR_ROOT: &str = "/tmp/controlzones";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    pub name: String,
    pub workdir: String,
    pub share_folder: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct OS {
    pub kernel: String,
    pub initram_fs: Option<String>,
    pub rootfs: String,
    pub kcmdline: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    pub cpuset: String,
    pub cpus: Option<Vec<u32>>,
    pub memory: u32,
    pub share_path: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlZone {
    pub meta: Option<Meta>,
    pub os: OS,
    pub resource: Resource,
}

fn parse_cpuset(cpuset_config: &str) -> BTreeSet<u32> {
    let mut cpus = Vec::new();

    for part in cpuset_config.split(',') {
        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() == 2 {
                if let (Result::Ok(start), Result::Ok(end)) =
                    (range[0].parse::<u32>(), range[1].parse::<u32>())
                {
                    cpus.extend(start..=end);
                }
            }
        } else if let Result::Ok(num) = part.parse::<u32>() {
            cpus.push(num);
        }
    }

    BTreeSet::from_iter(cpus.into_iter())
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlZoneInner {
    pub meta: Meta,
    pub os: OS,
    pub resource: ResourceInner,
}

impl ControlZoneInner {
    pub fn new_from_config(file: &PathBuf) -> anyhow::Result<Self> {
        let config = fs::read_to_string(file)?;
        let cz: ControlZone = serde_yaml::from_str(&config)?;

        // init meta

        let meta = match cz.meta {
            Some(meta) => meta,
            None => {
                let fname = file
                    .file_name()
                    .expect("not a valid file name")
                    .to_str()
                    .expect("filename convert to str failed");

                let name = if fname.ends_with(".yaml") {
                    fname[..fname.len() - 5].to_owned()
                } else {
                    fname.to_owned()
                };

                let workdir = PathBuf::from(WORKDIR_ROOT).join(PathBuf::from(&name));

                let share_folder = workdir.join(PathBuf::from("controlzone"));

                let workdir_str = workdir
                    .to_str()
                    .ok_or(anyhow!("parse workdir failed"))?
                    .to_owned();
                let share_folder_str = share_folder
                    .to_str()
                    .ok_or(anyhow!("parse share_folder failed"))?
                    .to_owned();

                Meta {
                    name: name,
                    workdir: workdir_str,
                    share_folder: share_folder_str,
                }
            }
        };

        // init resource
        let cpus = match cz.resource.cpus {
            Some(cpus) => cpus,
            None => parse_cpuset(&cz.resource.cpuset).into_iter().collect(),
        };

        let resource = ResourceInner {
            cpus,
            memory: cz.resource.memory,
        };

        Ok(Self {
            meta,
            os: cz.os,
            resource,
        })
    }

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

    /// delete workdir of controlzone
    pub fn delete_workdir(&self) {
        let workdir = PathBuf::from(&self.meta.workdir);
        if workdir.exists() {
            _ = fs::remove_dir_all(&workdir)
        }
    }

    /// init workdir for control zone
    /// copy image from src to des
    pub fn init_workdir(&mut self) -> anyhow::Result<()> {
        self.delete_workdir();

        let workdir = PathBuf::from(&self.meta.workdir);
        fs::create_dir_all(&workdir)?;

        // copy rootfs
        let src_rootfs = PathBuf::from(&self.os.rootfs);
        let des_rootfs = workdir.join(PathBuf::from("cz.img"));
        fs::copy(src_rootfs, &des_rootfs)?;

        // create sharefolder
        let share_folder = PathBuf::from(&self.meta.share_folder);
        fs::create_dir(&share_folder)?;

        self.os.rootfs = des_rootfs
            .to_str()
            .ok_or(anyhow!("parse rootfs failed"))?
            .to_owned();

        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceInner {
    pub cpus: Vec<u32>,
    pub memory: u32,
}

#[cfg(test)]
mod test {

    use std::collections::BTreeSet;

    use crate::control_zone::{ControlZoneInner, Meta, ResourceInner, OS};

    use super::parse_cpuset;

    const TARGET_XML: &str = "<domain type='kvm'>
<name>controlzone01</name>
<memory unit='MB'>4096</memory>
<vcpu placement='static'>4</vcpu>
<cputune>
<vcpupin vcpu='0' cpuset='130'/>
<vcpupin vcpu='1' cpuset='131'/>
<vcpupin vcpu='2' cpuset='132'/>
<vcpupin vcpu='3' cpuset='133'/>
</cputune>
<os>
<type arch='x86_64' machine='pc-i440fx-jammy'>hvm</type>
<kernel>/tmp/control_zone/kernels/cfs-virt</kernel>
<initrd>/tmp/control_zone/initramfs-virt</initrd>
<cmdline>vmlinuz-virt initrd=initramfs-virt root=LABEL=root rootfstype=ext4 modules=kms,scsi,virtio console=ttyS0</cmdline>
<boot dev='hd'/>
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
<source file='/tmp/control_zone/images/alpine-uefi.qcow2'/>
#<target dev='vda' bus='virtio'/>
<alias name='ua-box-volume-0'/>
<address type='pci' domain='0x0000' bus='0x00' slot='0x02' function='0x0'/>
</disk>
<interface type='network'>
<domain name='controlzone01'/>
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
<source dir='/tmp/control_zone/controlzone'/>
<target dir='hostshare'/>
<address type='pci' domain='0x0000' bus='0x00' slot='0x06' function='0x0'/>
</filesystem>
</devices>
</domain>";

    #[test]
    fn test_to_xml() {
        let controlzone = ControlZoneInner {
            meta: Meta{
                name: String::from("controlzone01"),
                workdir: String::from("/tmp/control_zone/"),
                share_folder: String::from("/tmp/control_zone/controlzone"),
            },
            os: OS{
                kernel: String::from("/tmp/control_zone/kernels/cfs-virt"),
                initram_fs: Some(String::from("/tmp/control_zone/initramfs-virt")),
                rootfs: String::from("/tmp/control_zone/images/alpine-uefi.qcow2"),
                kcmdline: String::from("vmlinuz-virt initrd=initramfs-virt root=LABEL=root rootfstype=ext4 modules=kms,scsi,virtio console=ttyS0"),
            },
            resource: ResourceInner{
                cpus: vec![130, 131, 132, 133],
                memory:4096,
            },
        };

        let xml = controlzone.to_xml().unwrap();
        let yaml = serde_yaml::to_string(&controlzone).unwrap();
        println!("{}", yaml);
        assert_eq!(xml, TARGET_XML)
    }

    #[test]
    fn test_parse_cpuset() {
        let cpu_set = "0,3";
        let cpus = parse_cpuset(cpu_set);
        assert_eq!(cpus, BTreeSet::from_iter(vec![0, 3]));

        let cpu_set = "0-3";
        let cpus = parse_cpuset(cpu_set);
        assert_eq!(cpus, BTreeSet::from_iter(vec![0, 1, 2, 3]));

        let cpu_set = "0-3,4,6-7";
        let cpus = parse_cpuset(cpu_set);
        assert_eq!(cpus, BTreeSet::from_iter(vec![0, 1, 2, 3, 4, 6, 7]));

        let cpu_set = "0-3, 0-3";
        let cpus = parse_cpuset(cpu_set);
        assert_eq!(cpus, BTreeSet::from_iter(vec![0, 1, 2, 3]));
    }
}

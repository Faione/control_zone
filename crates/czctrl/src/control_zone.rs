use anyhow::Ok;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlZone {
    name: String,
    guestos: GuestOS,
    resource: Resource,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GuestOS {
    kernel: String,
    initrd: String,
    rootfs: String,
    share_path: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    cpuset: Option<String>,
    cpus: Option<usize>,
    memory: u32,
}

impl ControlZone {
    pub fn to_xml(&self) -> anyhow::Result<String> {
        let mut buf = String::from("<domain type='kvm'>\n");

        writeln!(&mut buf, "<name>{}</name>", self.name)?;
        writeln!(
            &mut buf,
            "<memory unit='KiB'>{}</memory>",
            self.resource.memory
        )?;

        let mut cpu_nums = self.resource.cpus.unwrap_or_default();

        let cpuset = self.resource.cpuset.as_ref().and_then(|cpuset| {
            let cpus = parse_cpuset(&cpuset);
            cpu_nums = cpus.len();
            Some(cpus)
        });

        match cpuset {
            Some(cpus) => {
                writeln!(&mut buf, "<vcpu placement='static'>{}</vcpu>", cpu_nums)?;
                writeln!(&mut buf, "<cputune>")?;

                for (i, cpu) in cpus.iter().enumerate() {
                    writeln!(&mut buf, "<vcpupin vcpu='{}' cpuset='{}'/>", i, cpu)?;
                }
            }
            None => todo!(),
        }
        writeln!(&mut buf, "</cputune>")?;

        writeln!(
            &mut buf,
            "<os>\n<type arch='x86_64' machine='pc-i440fx-jammy'>hvm</type>"
        )?;
        writeln!(&mut buf, "<kernel>{}</kernel>", self.guestos.kernel)?;
        writeln!(&mut buf, "<initrd>{}</initrd>", self.guestos.initrd)?;

        writeln!(&mut buf, "<cmdline>vmlinuz-virt initrd=initramfs-virt root=LABEL=root rootfstype=ext4 modules=kms,scsi,virtio console=ttyS0</cmdline>")?;

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
<source dir='/tmp/control_zone/controlzone'/>
<target dir='hostshare'/>
<address type='pci' domain='0x0000' bus='0x00' slot='0x06' function='0x0'/>
</filesystem>
</devices>
</domain>",
            self.guestos.rootfs, self.name,
        )?;

        Ok(buf)
    }
}

fn parse_cpuset(cpuset: &str) -> Vec<i32> {
    let mut cpus = Vec::new();

    for part in cpuset.split(',') {
        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() == 2 {
                if let (Result::Ok(start), Result::Ok(end)) =
                    (range[0].parse::<i32>(), range[1].parse::<i32>())
                {
                    cpus.extend(start..=end);
                }
            }
        } else if let Result::Ok(num) = part.parse::<i32>() {
            cpus.push(num);
        }
    }

    cpus
}

#[cfg(test)]
mod test {
    use serde::Serialize;

    use super::{parse_cpuset, ControlZone, GuestOS, Resource};

    const TARGET_XML: &str = "<domain type='kvm'>
<name>controlzone01</name>
<memory unit='KiB'>4194304</memory>
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
        let cz = ControlZone {
            name: String::from("controlzone01"),
            guestos: GuestOS {
                kernel: String::from("/tmp/control_zone/kernels/cfs-virt"),
                initrd: String::from("/tmp/control_zone/initramfs-virt"),
                rootfs: String::from("/tmp/control_zone/images/alpine-uefi.qcow2"),
                share_path: String::from("/tmp/control_zone/controlzone"),
            },
            resource: Resource {
                cpuset: Some(String::from("130-133")),
                cpus: None,
                memory: 4194304,
            },
        };

        let xml = cz.to_xml().unwrap();
        let yaml = serde_yaml::to_string(&cz).unwrap();
        println!("{}", yaml);
        assert_eq!(xml, TARGET_XML)
    }

    #[test]
    fn test_parse_cpuset() {
        let cpu_set = "0,3";
        let cpus = parse_cpuset(cpu_set);
        assert_eq!(cpus, vec![0, 3]);

        let cpu_set = "0-3";
        let cpus = parse_cpuset(cpu_set);
        assert_eq!(cpus, vec![0, 1, 2, 3]);

        let cpu_set = "0-3,4,6-7";
        let cpus = parse_cpuset(cpu_set);
        assert_eq!(cpus, vec![0, 1, 2, 3, 4, 6, 7]);
    }
}

use std::collections::BTreeSet;

use crate::controlzone::{util::parse_cpuset, ControlZone, Meta, Resource, CZOS};

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
    let controlzone = ControlZone {
        meta: Meta{
            name: String::from("controlzone01"),
            workdir: String::from("/tmp/control_zone/"),
            share_folder: String::from("/tmp/control_zone/controlzone"),
            full_config: String::from("nothing"),
        },
        os: CZOS{
            kernel: String::from("/tmp/control_zone/kernels/cfs-virt"),
            initram_fs: Some(String::from("/tmp/control_zone/initramfs-virt")),
            rootfs: String::from("/tmp/control_zone/images/alpine-uefi.qcow2"),
            kcmdline: String::from("vmlinuz-virt initrd=initramfs-virt root=LABEL=root rootfstype=ext4 modules=kms,scsi,virtio console=ttyS0"),
        },
        resource: Resource{
            cpus: vec![130, 131, 132, 133],
            memory:4096,
            cpuset: String::from("nothing"),
        },
        state: crate::controlzone::State::Created,
        
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

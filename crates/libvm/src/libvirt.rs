//! An abstraction on top of the libvirt bindings.
use anyhow::{anyhow, bail, Ok};
use libcz::{vruntime::VRuntime, ControlZone};
use log::debug;
use std::fmt::Write;
use virt::{connect::Connect, domain::Domain, sys::VIR_DOMAIN_INTERFACE_ADDRESSES_SRC_LEASE};

/// Errors from this module.
#[derive(Debug, thiserror::Error)]
pub enum VirtError {
    /// Error connecting to libvirtd.
    #[error("couldn't connect to the libvirt daemon")]
    Connect(#[source] virt::error::Error),

    /// Error doing I/O.
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

/// Access libvirt for all the things this program needs.
pub struct Libvirt {
    conn: Connect,
}

impl Libvirt {
    pub fn connect(url: &str) -> anyhow::Result<Self> {
        debug!("connecting to libvirtd {}", url);
        let conn = Connect::open(url).map_err(VirtError::Connect)?;
        Ok(Self { conn })
    }
}

pub fn cz_to_xml(cz: &ControlZone) -> anyhow::Result<String> {
    let mut buf = String::from("<domain type='kvm'>\n");

    // Init name
    writeln!(&mut buf, "<name>{}</name>", cz.meta.name)?;

    // Init memory
    writeln!(
        &mut buf,
        "<memory unit='MB'>{}</memory>",
        cz.resource.memory
    )?;

    // Init static CPU
    writeln!(
        &mut buf,
        "<vcpu placement='static'>{}</vcpu>",
        cz.resource.cpus.len()
    )?;
    writeln!(&mut buf, "<cputune>")?;
    for (i, cpu) in cz.resource.cpus.iter().enumerate() {
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
        cz.os.rootfs
    );

    // Init Network
    // if static ip configured, then only using bridge network
    let network = match &cz.resource.static_net {
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
            cz.meta.name
        ),
    };

    // Init Sharefolder
    let sharefolder = format!(
        "\
        <filesystem type='mount' accessmode='mapped'>\n\
        <source dir='{}'/>\n\
        <target dir='hostshare'/>\n\
        <address type='pci' domain='0x0000' bus='0x00' slot='0x06' function='0x0'/>\n\
        </filesystem>",
        cz.meta.share_folder
    );

    // Init OS
    writeln!(
        &mut buf,
        "<os>\n<type arch='x86_64' machine='pc-i440fx-jammy'>hvm</type>"
    )?;
    writeln!(&mut buf, "<kernel>{}</kernel>", cz.os.kernel)?;
    if let Some(initrd) = &cz.os.initram_fs {
        writeln!(&mut buf, "<initrd>{}</initrd>", initrd)?;
    }
    writeln!(&mut buf, "<cmdline>{}</cmdline>", cz.os.kcmdline)?;

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

fn first_ip(domain: &Domain) -> anyhow::Result<String> {
    let interfaces = domain.interface_addresses(VIR_DOMAIN_INTERFACE_ADDRESSES_SRC_LEASE, 0)?;

    if interfaces.len() == 0 {
        bail!("no interface found for control zone")
    }

    let addr = &interfaces[0].addrs;

    if addr.len() == 0 {
        bail!("no addr found for control zone")
    }

    Ok(addr[0].addr.clone())
}

impl VRuntime for Libvirt {
    fn start(&self, cz: &mut ControlZone) -> anyhow::Result<()> {
        let config = cz_to_xml(cz)?;
        Domain::create_xml(&self.conn, &config, 0)?;
        Ok(())
    }

    fn stop(&self, cz: &mut ControlZone) -> anyhow::Result<()> {
        let domain = Domain::lookup_by_name(&self.conn, &cz.meta.name)?;
        if let Err(e) = domain.destroy() {
            bail!("destroy control zone failed: {e}")
        }
        Ok(())
    }

    fn addi_bar(&self) {
        println!("{:6}{:16}", "ID", "IP");
    }

    fn addi_infoper(&self, cz: &ControlZone) -> anyhow::Result<()> {
        let domain = Domain::lookup_by_name(&self.conn, &cz.meta.name)?;
        let id = domain.get_id().ok_or(anyhow!("get id failed"))?;
        let ip = first_ip(&domain)?;
        println!("{:<6}{:16}", id, ip);
        Ok(())
    }
}

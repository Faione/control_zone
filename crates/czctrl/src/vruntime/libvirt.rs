use libcz::ControlZone;

use crate::config::DEFAUL_LIBVIRT_URI;

use super::{cz_wait_f, VRuntime};

pub fn vruntime() -> VRuntime {
    VRuntime {
        start_f: Box::new(libvirt_start_f),
        stop_f: Box::new(libvirt_stop_f),
        wait_f: Box::new(cz_wait_f),
    }
}

pub fn libvirt_start_f(cz: &ControlZone) -> anyhow::Result<()> {
    let virt_cli = libvm::virt::Libvirt::connect(DEFAUL_LIBVIRT_URI)?;
    virt_cli.create_control_zone(&cz.to_xml()?)?;

    Ok(())
}

pub fn libvirt_stop_f(cz: &ControlZone) -> anyhow::Result<()> {
    let virt_cli = libvm::virt::Libvirt::connect(DEFAUL_LIBVIRT_URI)?;
    let cz_wrapper = virt_cli.get_control_zone_by_name(&cz.meta.name)?;
    cz_wrapper.destroy()
}

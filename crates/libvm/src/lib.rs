use libcz::vruntime::DVRuntime;
use libvirt::Libvirt;

mod libvirt;
mod qemu;
#[cfg(test)]
mod test;

pub use libvirt::cz_to_xml;
use qemu::Qemu;

pub fn new_libvirt_vruntime(url: &str) -> DVRuntime {
    Box::new(Libvirt::new(url).unwrap())
}

pub fn new_qemu_vruntime() -> DVRuntime {
    Box::new(Qemu {})
}

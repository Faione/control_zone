use libvirt::Libvirt;

mod libvirt;
#[cfg(test)]
mod test;

pub use libvirt::cz_to_xml;

pub fn new_libvirt_vruntime(url: &str) -> Libvirt {
    Libvirt::connect(url).unwrap()
}

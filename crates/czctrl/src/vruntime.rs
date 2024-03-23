use crate::config::DEFAUL_LIBVIRT_URI;
use clap::ValueEnum;
use libcz::vruntime::DVRuntime;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum VRuntimeType {
    Libvirt,
    Qemu,
    CloudHyper,
}

impl From<VRuntimeType> for DVRuntime {
    fn from(t: VRuntimeType) -> Self {
        match t {
            VRuntimeType::Libvirt => Box::new(libvm::new_libvirt_vruntime(DEFAUL_LIBVIRT_URI)),
            VRuntimeType::Qemu => todo!(),
            VRuntimeType::CloudHyper => todo!(),
        }
    }
}

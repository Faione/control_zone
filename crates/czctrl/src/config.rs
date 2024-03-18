// apply
pub const DEFAUL_LIBVIRT_URI: &str = "qemu:///system";
pub const TRY_COUNT: i8 = 10;
pub const TRY_INTERVAL: u64 = 1; // try interval (second)

// observe

pub const RESCTL_ROOT: &str = "/sys/fs/resctrl";
pub const CGROUP_ROOT: &str = "/sys/fs/cgroup";

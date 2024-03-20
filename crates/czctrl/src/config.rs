// ControlZone
pub const WORKDIR_ROOT: &str = "/tmp/controlzones";
pub const CZ_CONFIG: &str = "controlzone.yaml";
pub const CZ_IMAGE: &str = "cz.img";

pub const POD_DIR: &str = "pod";

pub const INFO_DIR: &str = "info";
// sharefolder/info/state
pub const STATE_FILE: &str = "state";

// apply
pub const DEFAUL_LIBVIRT_URI: &str = "qemu:///system";
pub const TRY_COUNT: i8 = 10;
pub const TRY_INTERVAL: u64 = 1; // try interval (second)

// observe
pub const RESCTL_ROOT: &str = "/sys/fs/resctrl";
pub const CGROUP_ROOT: &str = "/sys/fs/cgroup";

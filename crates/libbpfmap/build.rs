use std::env;
use std::path::PathBuf;

use libbpf_cargo::SkeletonBuilder;

const SRC: &str = "src/bpf/cgroup_map.bpf.c";
const VMLINUX: &str = "include/vmlinux-5.15";

fn main() {
    let mut out =
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set in build script"));
    out.push("cgroup_map.skel.rs");
    SkeletonBuilder::new()
        .source(SRC)
        .clang_args(format!("-I {VMLINUX}"))
        .build_and_generate(&out)
        .unwrap();
    println!("cargo:rerun-if-changed={SRC}");
}

use std::collections::BTreeSet;

use crate::{resource::StaticNet, util::parse_cpuset};

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

#[test]
fn test_parse_static_net_cfg() {
    let static_net = StaticNet {
        address: String::from("192.168.1.10"),
        netmask: String::from("255.255.255.0"),
        gateway: String::from("192.168.1.2"),
    };

    let target_cfg = "iface lo inet loopback
iface eth0 inet static
    address 192.168.1.10
    netmask 255.255.255.0
    gateway 192.168.1.2";

    let cfg = static_net.to_interface_cfg().unwrap();

    assert_eq!(cfg, target_cfg)
}

mod cgroup_map {
    include!(concat!(env!("OUT_DIR"), "/cgroup_map.skel.rs"));
}

use std::os::unix::fs::MetadataExt;

use anyhow::{Ok, Result};
use cgroup_map::*;
use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    MapFlags,
};

#[inline]
fn read_cgroup_inode_id(path: &str) -> anyhow::Result<u64> {
    Ok(std::fs::metadata(path)?.ino())
}
pub struct CgroupMapWrapper<'a> {
    skel: CgroupMapSkel<'a>,
}

impl<'a> CgroupMapWrapper<'a> {
    pub fn new() -> Result<CgroupMapWrapper<'a>> {
        // load 返回的是具有所有权示例
        // map 返回的则是带有引用的所有权实例
        // 即所有的 map 都对 MapSkel 中的一部分产生了借用
        // 因此必须保证保证生命周期
        return Ok(CgroupMapWrapper {
            skel: CgroupMapSkelBuilder::default().open()?.load()?,
        });
    }

    /// list kv in bpf map
    pub fn list(&self) {
        let maps = self.skel.maps();
        let cgroup_map = maps.cgroup_map();

        cgroup_map.keys().for_each(|k| {
            let _ = cgroup_map
                .lookup(&k, MapFlags::ANY)
                .unwrap()
                .is_some_and(|val| {
                    let mut key: u64 = 0;
                    let mut cgroup_id: u64 = 0;
                    plain::copy_from_bytes(&mut key, &k).expect("data not long enough");
                    plain::copy_from_bytes(&mut cgroup_id, &val).expect("data not long enough");

                    println!("key: {}, val: {}", key, cgroup_id);
                    true
                });
        })
    }

    pub fn insert_list(&self, cgroup_paths: &Vec<String>) -> Result<()> {
        let maps = self.skel.maps();
        let cgroup_map = maps.cgroup_map();

        for path in cgroup_paths {
            let cgroup_id = read_cgroup_inode_id(path)?.to_le_bytes();
            cgroup_map.update(&cgroup_id, &cgroup_id, MapFlags::ANY)?;
        }
        Ok(())
    }
}

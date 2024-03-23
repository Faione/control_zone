use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::Parser;

use libcz::{
    vruntime::{addition_info_bar, addition_info_per, DVRuntime},
    ControlZone, CZ_CONFIG,
};

use crate::GloablOpts;

#[derive(Parser, Debug)]
pub struct List {
    /// List all Control Zones
    #[arg(short, long)]
    use_vruntime: bool,
}

pub fn list(args: List, global_opts: &GloablOpts) -> Result<()> {
    let root_dir = global_opts.root_dir();
    let controlzones: Vec<ControlZone> = fs::read_dir(root_dir)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }

            let full_config = path.join(CZ_CONFIG);
            if !full_config.exists() {
                return None;
            }

            ControlZone::new_from_full_config(&full_config).ok()
        })
        .collect();

    let vruntime: DVRuntime = global_opts.vruntime.into();
    print!("{:16}{:20}{:10}{:10}", "NAME", "KERNEL", "CPUS", "STATUS");

    if args.use_vruntime {
        vruntime.addi_bar()
    } else {
        addition_info_bar()
    };

    controlzones.iter().try_for_each(|cz| {
        let kernel_name = PathBuf::from(&cz.os.kernel)
            .file_name()
            .ok_or(anyhow!("parse controlzone name failed"))?
            .to_str()
            .ok_or(anyhow!("parse controlzone name failed"))?
            .to_owned();

        print!(
            "{:16}{:20}{:10}{:10}",
            cz.meta.name, kernel_name, cz.resource.cpuset, cz.state
        );

        if args.use_vruntime {
            vruntime.addi_infoper(cz)
        } else {
            addition_info_per(cz)
        }
    })
}

#[cfg(test)]
mod test {

    #[test]
    fn test_print() {
        println!(
            "{:16}{:10}{:10}{:10}{:6}{:16}",
            "NAME", "KERNEL", "CPUS", "STATUS", "ID", "IP"
        );

        println!(
            "{:16}{:10}{:10}{:10}{:6}{:16}",
            "test_cz", "cz_base", "100-124", "RUNNING", "10", "192.168.0.2"
        );

        print!("{:16}{:10}{:10}{:10}", "NAME", "KERNEL", "CPUS", "STATUS");
        println!("{:6}{:16}", "ID", "IP");

        print!(
            "{:16}{:10}{:10}{:10}",
            "test_cz", "cz_base", "100-124", "RUNNING"
        );

        println!("{:<6}{:16}", 10, "192.168.0.2")
    }
}

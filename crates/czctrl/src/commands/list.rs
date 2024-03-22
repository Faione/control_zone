use std::{fs, path::PathBuf};

use anyhow::{anyhow, Ok, Result};
use clap::Parser;
use libvm::virt;

use libcz::{ControlZone, CZ_CONFIG, INFO_DIR, IP_FILE};
use log::error;

use crate::{config::DEFAUL_LIBVIRT_URI, GloablOpts};

#[derive(Parser, Debug)]
pub struct List {
    /// List all Control Zones
    #[arg(short, long)]
    libvirt: bool,
}

enum AddtionInfo {
    Libvirt,
    Origin,
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

    let addtion_mode = if args.libvirt {
        AddtionInfo::Libvirt
    } else {
        AddtionInfo::Origin
    };

    print!("{:16}{:20}{:10}{:10}", "NAME", "KERNEL", "CPUS", "STATUS");
    let addtion_info_f: Box<dyn Fn(&ControlZone) -> Result<()>> = match addtion_mode {
        AddtionInfo::Libvirt => {
            println!("{:6}{:16}", "ID", "IP");

            let virt_cli = virt::Libvirt::connect(DEFAUL_LIBVIRT_URI)?;
            Box::new(move |controlzone: &ControlZone| -> Result<()> {
                let cz_wrapper = virt_cli.get_control_zone_by_name(&controlzone.meta.name)?;
                let id = cz_wrapper.get_id()?;
                let ip = cz_wrapper.get_ip()?;
                println!("{:<6}{:16}", id, ip);
                Ok(())
            })
        }
        AddtionInfo::Origin => {
            println!("{:16}", "IP");
            Box::new(|controlzone: &ControlZone| -> Result<()> {
                let ip_file = PathBuf::from(&controlzone.meta.share_folder)
                    .join(INFO_DIR)
                    .join(IP_FILE);

                if !ip_file.exists() {
                    error!("control zone may not initialized")
                }

                println!("{}", fs::read_to_string(ip_file)?);
                Ok(())
            })
        }
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
        addtion_info_f(cz)
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

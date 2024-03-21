use anyhow::{bail, Ok};

#[derive(Debug)]
pub struct Info {
    pub ip: String,
}

fn first_ip_on_first_nic() -> anyhow::Result<String> {
    if let Some(ipv4) = nix::ifaddrs::getifaddrs()?
        .filter_map(|ifaddr| ifaddr.address)
        .filter_map(|addr| {
            if let Some(ipv4_addr) = addr.as_sockaddr_in() {
                Some(ipv4_addr.ip())
            } else {
                None
            }
        })
        .find(|ip| !ip.is_broadcast() && !ip.is_link_local() && !ip.is_loopback())
    {
        Ok(ipv4.to_string())
    } else {
        bail!("no valied ip found")
    }
}

pub fn fetch_info() -> anyhow::Result<Info> {
    // first ip on first nic

    let ip = first_ip_on_first_nic()?;

    Ok(Info { ip })
}

#[cfg(test)]
mod test {
    use super::first_ip_on_first_nic;

    #[test]
    fn test_first_nic() {
        let ip = first_ip_on_first_nic().unwrap();
        println!("{ip}")
    }
}

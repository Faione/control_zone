//! An abstraction on top of the libvirt bindings.
use anyhow::{anyhow, bail, Ok, Result};
use log::debug;
use std::path::PathBuf;
use virt::{connect::Connect, domain::Domain, sys::VIR_DOMAIN_INTERFACE_ADDRESSES_SRC_LEASE};

/// Errors from this module.
#[derive(Debug, thiserror::Error)]
pub enum VirtError {
    /// Error connecting to libvirtd.
    #[error("couldn't connect to the libvirt daemon")]
    Connect(#[source] virt::error::Error),

    /// Error listing domains.
    #[error("couldn't list all domains")]
    Domains(#[source] virt::error::Error),

    /// Error listing domains.
    #[error("couldn't get name of domain")]
    GetName(#[source] virt::error::Error),

    /// Error checking if domain is active.
    #[error("couldn't check is domain {0} is active")]
    IsActive(String, #[source] virt::error::Error),

    /// Error getting domain's XML description.
    #[error("couldn't get domain's XML description: {0}")]
    GetXml(String, #[source] virt::error::Error),

    /// Error detaching cloud-init ISO from domain
    #[error("couldn't detach cloud-init ISO file from domain {0}")]
    DetachIso(String, #[source] virt::error::Error),

    /// Error detaching drive from domain
    #[error("couldn't create domain {0}")]
    Create(String, #[source] virt::error::Error),

    /// Error shutting down domain
    #[error("couldn't shut down domain {0}")]
    Shutdown(String, #[source] virt::error::Error),

    /// Error undefining domain
    #[error("couldn't undefine domain {0}")]
    Undefine(String, #[source] virt::error::Error),

    /// Error undefining domain
    #[error("couldn't set domain {0} to be autostarted")]
    Autostart(String, #[source] virt::error::Error),

    /// Failed to delete image file.
    #[error("failed to delete image file {0}")]
    DeleteImage(PathBuf, #[source] std::io::Error),

    /// Error doing I/O.
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

/// Access libvirt for all the things this program needs.
pub struct Libvirt {
    conn: Connect,
}

pub struct ControlZone {
    pub id: u32,
    pub name: String,
}

impl Libvirt {
    pub fn connect(url: &str) -> Result<Self> {
        debug!("connecting to libvirtd {}", url);
        let conn = Connect::open(url).map_err(VirtError::Connect)?;
        Ok(Self { conn })
    }

    pub fn define_control_zone(&self, config: &str) -> Result<ControlZoneWrapper> {
        let domain = Domain::define_xml(&self.conn, config)?;
        Ok(ControlZoneWrapper { domain })
    }

    pub fn create_control_zone(&self, config: &str) -> Result<ControlZoneWrapper> {
        let domain = Domain::create_xml(&self.conn, config, 0)?;
        Ok(ControlZoneWrapper { domain })
    }

    pub fn get_control_zone_by_id(&self, id: u32) -> Result<ControlZoneWrapper> {
        let domain = Domain::lookup_by_id(&self.conn, id)?;
        Ok(ControlZoneWrapper { domain })
    }

    pub fn get_control_zone_by_name(&self, name: &str) -> Result<ControlZoneWrapper> {
        let domain = Domain::lookup_by_name(&self.conn, name)?;
        Ok(ControlZoneWrapper { domain })
    }

    pub fn get_control_zones(&self) -> Result<Vec<ControlZone>> {
        Ok(self
            .conn
            .list_all_domains(0)
            .map_err(VirtError::Domains)?
            .into_iter()
            .filter_map(|domain| {
                let Some(name) = domain.get_name().ok() else {
                    return None;
                };

                let Some(id) = domain.get_id() else {
                    return None;
                };
                Some(ControlZone { id, name })
            })
            .collect())
    }

    pub fn get_control_zone_wrappers(&self) -> Result<Vec<ControlZoneWrapper>> {
        Ok(self
            .conn
            .list_all_domains(0)
            .map_err(VirtError::Domains)?
            .into_iter()
            .filter_map(|domain| Some(ControlZoneWrapper { domain }))
            .collect())
    }
}

pub struct ControlZoneWrapper {
    domain: Domain,
}

impl ControlZoneWrapper {
    pub fn new(domain: Domain) -> ControlZoneWrapper {
        ControlZoneWrapper { domain }
    }

    pub fn destroy(&self) -> Result<()> {
        if let Err(e) = self.domain.destroy() {
            Err(anyhow!("destroy control zone failed: {e}"))
        } else {
            Ok(())
        }
    }

    pub fn undefine(&self) -> Result<()> {
        if let Err(e) = self.domain.undefine() {
            Err(anyhow!("undefine control zone failed: {e}"))
        } else {
            Ok(())
        }
    }

    pub fn get_id(&self) -> Result<u32> {
        self.domain.get_id().ok_or(anyhow!("get id failed"))
    }

    pub fn get_name(&self) -> Result<String> {
        Ok(self.domain.get_name()?)
    }

    /// get the first interface's first addr of a exist control zone
    pub fn get_ip(&self) -> Result<String> {
        let interfaces = self
            .domain
            .interface_addresses(VIR_DOMAIN_INTERFACE_ADDRESSES_SRC_LEASE, 0)?;

        if interfaces.len() == 0 {
            bail!("no interface found for control zone")
        }

        let addr = &interfaces[0].addrs;

        if addr.len() == 0 {
            bail!("no addr found for control zone")
        }

        Ok(addr[0].addr.clone())
    }
}

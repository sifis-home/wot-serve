//! Service Advertisement
//!
//! Web Of Things (WoT) [Discovery](https://www.w3.org/TR/wot-discovery/) lists a number of means
//! for a Thing to advertise its presence.
//!
//! This implementation mainly focuses on [DNS-SD](https://www.w3.org/TR/wot-discovery/#introduction-dns-sd).

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::ops::Not;

use mdns_sd::{ServiceDaemon, ServiceInfo};

/// Error type for the module
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// mDNS internal error
    #[error("mdns internal error {0}")]
    Mdns(#[from] mdns_sd::Error),
    /// Network-specific error
    #[error("I/O error {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for the module
pub type Result<T> = std::result::Result<T, Error>;

/// Type of Thing being published
#[derive(Debug, Clone, Default, PartialEq, Eq, Copy)]
pub enum ThingType {
    /// Normal `Thing`.
    #[default]
    Thing,
    /// Thing Directory
    ///
    /// A `Thing` hosting a directory of `Thing`s.
    Directory,
}

impl ThingType {
    fn to_service_type(self) -> &'static str {
        use ThingType::*;
        match self {
            Thing => "_wot",
            Directory => "_directory._sub._wot",
        }
    }
    fn to_dns_type(self) -> &'static str {
        use ThingType::*;
        match self {
            Thing => "Thing",
            Directory => "Directory",
        }
    }
}

/// Service advertiser
///
/// A thing may be Advertised through a number of different means,
/// the current implementation uses only mdns-sd.
pub struct Advertiser {
    pub(crate) mdns: ServiceDaemon,
    /// Default set of ips for the system
    ips: Vec<Ipv4Addr>,
    /// Default hostname
    hostname: String,
}

const WELL_KNOWN: &str = "/.well-known/wot";

/// Builder to create a service
///
/// Call [`ServiceBuilder::build`] to publish it.
pub struct ServiceBuilder<'a> {
    mdns: &'a ServiceDaemon,
    ips: Vec<Ipv4Addr>,
    hostname: String,
    ty: ThingType,
    port: u16,
    path: String,
    name: String,
}

impl<'a> ServiceBuilder<'a> {
    fn new(ad: &'a Advertiser, name: impl Into<String>) -> ServiceBuilder<'a> {
        Self {
            name: name.into(),
            mdns: &ad.mdns,
            ips: ad.ips.clone(),
            hostname: ad.hostname.clone(),
            ty: ThingType::Thing,
            port: 8080,
            path: WELL_KNOWN.to_string(),
        }
    }

    /// Set the type between Thing and Directory
    ///
    /// A Directory has a specific `_directory` DNS-SD Subtype.
    pub fn thing_type(mut self, ty: ThingType) -> Self {
        self.ty = ty;

        self
    }

    /// The listening port for the advertised `Thing`.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;

        self
    }

    /// Where to find the Thing Description
    ///
    /// By default `/.well-known/wot` is used.
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();

        self
    }

    /// The hostname used by the MDNS daemon.
    pub fn hostname(mut self, host: impl Into<String>) -> Self {
        self.hostname = host.into();

        self
    }

    /// Listening IPs
    ///
    /// By default all the non-loopback ipv4 interfaces are used.
    pub fn ips<I: Into<Ipv4Addr>>(mut self, ips: impl Iterator<Item = I>) -> Self {
        self.ips = ips.map(|ip| ip.into()).collect();

        self
    }

    /// Consume the builder and register the service.
    pub fn build(self) -> Result<()> {
        let Self {
            mdns,
            ips,
            hostname,
            ty,
            path,
            port,
            name,
        } = self;

        let service_type = ty.to_service_type();
        let domain = format!("{service_type}._tcp.local.");
        let mut props = HashMap::new();

        props.insert("td".to_string(), path);
        props.insert("type".to_string(), ty.to_dns_type().to_string());

        let service = ServiceInfo::new(
            &domain,
            name.as_ref(),
            &hostname,
            ips.as_slice(),
            port,
            Some(props),
        )?;

        mdns.register(service)?;

        Ok(())
    }
}

impl Advertiser {
    /// Create a new service advertiser.
    pub fn new() -> Result<Self> {
        let mdns = ServiceDaemon::new()?;

        let mut hostname = hostname::get()?.to_string_lossy().to_string();
        if !hostname.ends_with(".local") {
            hostname.push_str(".local");
        }

        let ips = if_addrs::get_if_addrs()?
            .iter()
            .filter(|iface| iface.is_loopback().not())
            .filter_map(|iface| {
                let ip = iface.ip();
                match ip {
                    IpAddr::V4(ip) => Some(ip),
                    _ => None,
                }
            })
            .collect();

        let sa = Self {
            mdns,
            ips,
            hostname,
        };

        Ok(sa)
    }

    /// Register and Advertise a new service.
    pub fn add_service(&self, name: impl Into<String>) -> ServiceBuilder {
        ServiceBuilder::new(self, name)
    }
}

#[cfg(all(test, not(miri)))]
mod test {
    use super::*;

    use mdns_sd::{ServiceEvent::*, ServiceInfo};
    use std::time::Duration;

    #[test]
    fn set_hostname() {
        test_feature(
            "TestLampHostname",
            "_wot._tcp.local.",
            |b| b.hostname("testhost"),
            |info| {
                let props = info.get_properties();
                assert_eq!(props.get_property_val("td"), Some(WELL_KNOWN));
                assert_eq!(props.get_property_val("type"), Some("Thing"));
                assert_eq!(info.get_hostname(), "testhost.");
            },
        );
    }

    #[test]
    fn set_path() {
        test_feature(
            "TestLampPath",
            "_wot._tcp.local.",
            |b| b.path("/test/path"),
            |info| {
                let props = info.get_properties();
                assert_eq!(props.get_property_val("td"), Some("/test/path"));
                assert_eq!(props.get_property_val("type"), Some("Thing"));
            },
        );
    }

    #[test]
    fn set_type() {
        test_feature(
            "TestDirectory",
            "_directory._sub._wot._tcp.local.",
            |b| b.thing_type(ThingType::Directory),
            |info| {
                let props = info.get_properties();
                assert_eq!(props.get_property_val("td"), Some(WELL_KNOWN));
                assert_eq!(props.get_property_val("type"), Some("Directory"));
            },
        );
    }

    fn test_feature<F>(name: &str, browse: &str, build: F, check: fn(ServiceInfo))
    where
        F: for<'b> Fn(ServiceBuilder<'b>) -> ServiceBuilder<'b>,
    {
        let ad = Advertiser::new().unwrap();

        build(ad.add_service(name)).build().unwrap();

        let browser = ad.mdns.browse(browse).unwrap();

        while let Ok(ev) = browser.recv_timeout(Duration::from_secs(1)) {
            if let ServiceResolved(info) = ev {
                if info.get_fullname().split_once('.').unwrap().0 == name {
                    check(info);

                    return;
                }
            }
        }

        panic!("Thing not found");
    }
}

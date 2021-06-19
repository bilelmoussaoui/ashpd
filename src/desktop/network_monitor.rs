//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::network_monitor::NetworkMonitorProxy;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = NetworkMonitorProxy::new(&connection).await?;
//!
//!     println!("{}", proxy.can_reach("www.google.com", 80).await?);
//!
//!     println!("{}", proxy.get_available().await?);
//!
//!     println!("{:#?}", proxy.get_connectivity().await?);
//!
//!     println!("{}", proxy.get_metered().await?);
//!
//!     println!("{:#?}", proxy.get_status().await?);
//!
//!     Ok(())
//! }
//! ```
use crate::{
    helpers::{call_method, property},
    Error,
};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt;
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// The network status, composed of the availability, metered & connectivity
pub struct NetworkStatus {
    /// Whether the network is considered available.
    pub available: bool,
    /// Whether the network is considered metered.
    pub metered: bool,
    /// More detailed information about the host's network connectivity
    pub connectivity: Connectivity,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
/// Host's network activity
pub enum Connectivity {
    /// The host is not configured with a route to the internet.
    Local = 1,
    /// The host is connected to a network, but can't reach the full internet.
    Limited = 2,
    /// The host is behind a captive portal and cannot reach the full internet.
    CaptivePortal = 3,
    /// The host connected to a network, and can reach the full internet.
    FullNetwork = 4,
}

impl fmt::Display for Connectivity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let connectivity = match self {
            Self::Local => "local",
            Self::Limited => "limited",
            Self::CaptivePortal => "captive portal",
            Self::FullNetwork => "full network",
        };
        f.write_str(connectivity)
    }
}

/// The interface provides network status information to sandboxed applications.
/// It is not a portal in the strict sense, since it does not involve user
/// interaction. Applications are expected to use this interface indirectly,
/// via a library API such as the GLib `GNetworkMonitor` interface.
pub struct NetworkMonitorProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> NetworkMonitorProxy<'a> {
    /// Create a new instance of [`NetworkMonitorProxy`].
    pub async fn new(
        connection: &zbus::azync::Connection,
    ) -> Result<NetworkMonitorProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.NetworkMonitor")
            .path("/org/freedesktop/portal/desktop")?
            .destination("org.freedesktop.portal.Desktop")
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Returns whether the given hostname is believed to be reachable.
    ///
    /// # Arguments
    ///
    /// * `hostname` - The hostname to reach.
    /// * `port` - The port to reach.
    pub async fn can_reach(&self, hostname: &str, port: u32) -> Result<bool, Error> {
        call_method(&self.0, "CanReach", &(hostname, port)).await
    }

    /// Returns whether the network is considered available.
    /// That is, whether the system as a default route for at least one of IPv4
    /// or IPv6.
    pub async fn get_available(&self) -> Result<bool, Error> {
        call_method(&self.0, "GetAvailable", &()).await
    }

    /// Returns more detailed information about the host's network connectivity
    pub async fn get_connectivity(&self) -> Result<Connectivity, Error> {
        call_method(&self.0, "GetConnectivity", &()).await
    }

    /// Returns whether the network is considered metered.
    /// That is, whether the system as traffic flowing through the default
    /// connection that is subject to limitations by service providers.
    pub async fn get_metered(&self) -> Result<bool, Error> {
        call_method(&self.0, "GetMetered", &()).await
    }

    /// Returns the three values all at once.
    pub async fn get_status(&self) -> Result<NetworkStatus, Error> {
        call_method(&self.0, "GetStatus", &()).await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}

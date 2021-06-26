//! **Note** this portal doesn't work for sandboxed applications.
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
//!     println!("{}", proxy.is_available().await?);
//!
//!     println!("{:#?}", proxy.connectivity().await?);
//!
//!     println!("{}", proxy.is_metered().await?);
//!
//!     println!("{:#?}", proxy.status().await?);
//!
//!     Ok(())
//! }
//! ```

use super::{DESTINATION, PATH};
use crate::{
    helpers::{call_method, receive_signal},
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
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.NetworkMonitor`](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html#gdbus-org.freedesktop.portal.NetworkMonitor).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.NetworkMonitor")]
pub struct NetworkMonitorProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> NetworkMonitorProxy<'a> {
    /// Create a new instance of [`NetworkMonitorProxy`].
    pub async fn new(
        connection: &zbus::azync::Connection,
    ) -> Result<NetworkMonitorProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.NetworkMonitor")
            .path(PATH)?
            .destination(DESTINATION)
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::azync::Proxy<'_> {
        &self.0
    }

    /// Returns whether the given hostname is believed to be reachable.
    ///
    /// # Arguments
    ///
    /// * `hostname` - The hostname to reach.
    /// * `port` - The port to reach.
    #[doc(alias = "CanReach")]
    pub async fn can_reach(&self, hostname: &str, port: u32) -> Result<bool, Error> {
        call_method(&self.0, "CanReach", &(hostname, port)).await
    }

    /// Returns whether the network is considered available.
    /// That is, whether the system as a default route for at least one of IPv4
    /// or IPv6.
    #[doc(alias = "GetAvailable")]
    #[doc(alias = "get_available")]
    pub async fn is_available(&self) -> Result<bool, Error> {
        call_method(&self.0, "GetAvailable", &()).await
    }

    /// Returns more detailed information about the host's network connectivity
    #[doc(alias = "GetConnectivity")]
    #[doc(alias = "get_connectivity")]
    pub async fn connectivity(&self) -> Result<Connectivity, Error> {
        call_method(&self.0, "GetConnectivity", &()).await
    }

    /// Returns whether the network is considered metered.
    /// That is, whether the system as traffic flowing through the default
    /// connection that is subject to limitations by service providers.
    #[doc(alias = "GetMetered")]
    #[doc(alias = "get_metered")]
    pub async fn is_metered(&self) -> Result<bool, Error> {
        call_method(&self.0, "GetMetered", &()).await
    }

    /// Returns the three values all at once.
    #[doc(alias = "GetStatus")]
    #[doc(alias = "get_status")]
    pub async fn status(&self) -> Result<NetworkStatus, Error> {
        call_method(&self.0, "GetStatus", &()).await
    }

    /// Emitted when the network configuration changes.
    pub async fn receive_changed(&self) -> Result<(), Error> {
        receive_signal(&self.0, "changed").await
    }
}

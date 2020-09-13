//! # Examples
//!
//! ```no_run
//! use ashpd::desktop::network_monitor::NetworkMonitorProxy;
//! use zbus::fdo::Result;
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = NetworkMonitorProxy::new(&connection)?;
//!
//!     println!("{}", proxy.can_reach("www.google.com", 80)?);
//!
//!     println!("{}", proxy.get_available()?);
//!
//!     println!("{:#?}", proxy.get_connectivity()?);
//!
//!     println!("{}", proxy.get_metered()?);
//!
//!     println!("{:#?}", proxy.get_status()?);
//!
//!     Ok(())
//! }
//! ```
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// The network status, composed of the avaiability, metered & connectivity
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

#[dbus_proxy(
    interface = "org.freedesktop.portal.NetworkMonitor",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface provides network status information to sandboxed applications.
/// It is not a portal in the strict sense, since it does not involve user interaction.
/// Applications are expected to use this interface indirectly, via a library API such as the GLib GNetworkMonitor interface.
trait NetworkMonitor {
    /// Returns whether the given hostname is believed to be reachable
    ///
    /// # Arguments
    ///
    /// * `hostname` - The hostname to reach
    /// * `port` - The port to reach
    fn can_reach(&self, hostname: &str, port: u32) -> Result<bool>;

    /// Returns whether the network is considered available.
    /// That is, whether the system as a default route for at least one of IPv4 or IPv6.
    fn get_available(&self) -> Result<bool>;

    /// Returns more detailed information about the host's network connectivity
    fn get_connectivity(&self) -> Result<Connectivity>;

    /// Returns whether the network is considered metered.
    /// That is, whether the system as traffic flowing through the default connection that is subject ot limitations by service providers.
    fn get_metered(&self) -> Result<bool>;

    /// Returns the three values all at once.
    fn get_status(&self) -> Result<NetworkStatus>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

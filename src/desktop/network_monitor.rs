use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{Signature, Type};

#[derive(Serialize, Debug, Copy, Clone)]
#[non_exhaustive]
pub enum Connectivity {
    /// The host is not configured with a route to the internet.
    Local,
    /// The host is connected to a network, but can't reach the full internet.
    Limited,
    /// The host is behind a captive portal and cannot reach the full internet.
    CaptivePortal,
    /// The host connected to a network, and can reach the full internet.
    FullNetwork,
    // Invalid value
    Unknown,
}

impl<'de> Deserialize<'de> for Connectivity {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        let response = match value {
            1 => Connectivity::Local,
            2 => Connectivity::Limited,
            3 => Connectivity::CaptivePortal,
            4 => Connectivity::FullNetwork,
            _ => Connectivity::Unknown,
        };
        Ok(response)
    }

    fn deserialize_in_place<D>(
        deserializer: D,
        place: &mut Self,
    ) -> std::result::Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        *place = Deserialize::deserialize(deserializer)?;
        Ok(())
    }
}

impl Type for Connectivity {
    fn signature() -> Signature<'static> {
        Signature::from_str_unchecked("u")
    }
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.NetworkMonitor",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
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
    ///
    /// * `available` - bool
    /// * `metered` - bool
    /// * `connectivity` - Connectivity
    ///
    fn get_status(&self) -> Result<HashMap<String, zvariant::OwnedValue>>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

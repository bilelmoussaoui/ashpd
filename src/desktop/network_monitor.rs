use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.NetworkMonitor",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait NetworkMonitor {
    /// CanReach method
    fn can_reach(&self, hostname: &str, port: u32) -> Result<bool>;

    /// GetAvailable method
    fn get_available(&self) -> Result<bool>;

    /// GetConnectivity method
    fn get_connectivity(&self) -> Result<u32>;

    /// GetMetered method
    fn get_metered(&self) -> Result<bool>;

    /// GetStatus method
    fn get_status(&self) -> Result<HashMap<String, zvariant::OwnedValue>>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

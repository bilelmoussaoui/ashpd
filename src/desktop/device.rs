use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Device",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Device {
    /// AccessDevice method
    fn access_device(
        &self,
        pid: u32,
        devices: &[&str],
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

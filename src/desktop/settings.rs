use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Settings",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Settings {
    /// Read method
    fn read(&self, namespace: &str, key: &str) -> Result<zvariant::OwnedValue>;

    /// ReadAll method
    fn read_all(
        &self,
        namespaces: &[&str],
    ) -> Result<HashMap<String, HashMap<String, zvariant::OwnedValue>>>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Settings",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Settings {
    /// Reads a single value. Returns an error on any unknown namespace or key.
    ///
    /// Returns the value `key` is to to as a `zvariant::OwnedValue`
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace to look up key in
    /// * `key` - The key to get
    ///
    fn read(&self, namespace: &str, key: &str) -> Result<zvariant::OwnedValue>;

    /// Reads a single value. Returns an error on any unknown namespace or key.
    ///
    /// Returns a `HashMap` of namespaces to its keys and values.
    ///
    /// # Arguments
    ///
    /// * `namespaces` - List of namespaces to filter results by.
    ///
    ///     If `namespaces` is an empty array or contains an empty string it matches all.
    ///     Globbing is supported but only for trailing sections, e.g. "org.example.*".
    ///
    fn read_all(
        &self,
        namespaces: &[&str],
    ) -> Result<HashMap<String, HashMap<String, zvariant::OwnedValue>>>;

    // TODO: re-enable once signals are available
    // fn setting_changed(&self, namespace: &str, key: &str, value: zvariant::OwnedValue);

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

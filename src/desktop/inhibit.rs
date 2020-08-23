use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Inhibit",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Inhibit {
    /// CreateMonitor method
    fn create_monitor(
        &self,
        window: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// Inhibit method
    fn inhibit(
        &self,
        window: &str,
        flags: u32,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// QueryEndResponse method
    fn query_end_response(&self, session_handle: &str) -> Result<()>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

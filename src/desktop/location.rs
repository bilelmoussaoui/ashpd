use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Location",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Location {
    /// CreateSession method
    fn create_session(&self, options: HashMap<&str, zvariant::Value>) -> Result<String>;

    /// Start method
    fn start(
        &self,
        session_handle: &str,
        parent_window: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Value;

#[dbus_proxy(
    interface = "org.freedesktop.portal.Location",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications query basic information about the location.
trait Location {
    /// CreateSession method
    fn create_session(&self, options: HashMap<&str, Value>) -> Result<String>;

    /// Start method
    fn start(
        &self,
        session_handle: &str,
        parent_window: &str,
        options: HashMap<&str, Value>,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

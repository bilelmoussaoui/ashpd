use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Print",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Print {
    /// PreparePrint method
    fn prepare_print(
        &self,
        parent_window: &str,
        title: &str,
        settings: HashMap<&str, zvariant::Value>,
        page_setup: HashMap<&str, zvariant::Value>,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// Print method
    fn print(
        &self,
        parent_window: &str,
        title: &str,
        fd: std::os::unix::io::RawFd,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

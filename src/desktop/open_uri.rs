use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.OpenURI",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait OpenURI {
    /// OpenDirectory method
    fn open_directory(
        &self,
        parent_window: &str,
        fd: std::os::unix::io::RawFd,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// OpenFile method
    fn open_file(
        &self,
        parent_window: &str,
        fd: std::os::unix::io::RawFd,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// OpenURI method
    fn open_uri(
        &self,
        parent_window: &str,
        uri: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

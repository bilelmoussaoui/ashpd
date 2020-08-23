use std::collections::HashMap;
use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Value;

#[dbus_proxy(
    interface = "org.freedesktop.portal.Print",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications print.
trait Print {
    /// PreparePrint method
    fn prepare_print(
        &self,
        parent_window: &str,
        title: &str,
        settings: HashMap<&str, Value>,
        page_setup: HashMap<&str, Value>,
        options: HashMap<&str, Value>,
    ) -> Result<String>;

    /// Print method
    fn print(
        &self,
        parent_window: &str,
        title: &str,
        fd: RawFd,
        options: HashMap<&str, Value>,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

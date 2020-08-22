use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.FileTransfer",
    default_path = "/org/freedesktop/portal/documents"
)]
trait FileTransfer {
    /// AddFiles method
    fn add_files(
        &self,
        key: &str,
        fds: &[std::os::unix::io::RawFd],
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<()>;

    /// RetrieveFiles method
    fn retrieve_files(
        &self,
        key: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<Vec<String>>;

    /// StartTransfer method
    fn start_transfer(&self, options: HashMap<&str, zvariant::Value>) -> Result<String>;

    /// StopTransfer method
    fn stop_transfer(&self, key: &str) -> Result<()>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

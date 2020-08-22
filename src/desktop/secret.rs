use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Secret",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Secret {
    /// RetrieveSecret method
    fn retrieve_secret(
        &self,
        fd: std::os::unix::io::RawFd,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

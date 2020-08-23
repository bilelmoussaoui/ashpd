use std::collections::HashMap;
use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Value;

#[dbus_proxy(
    interface = "org.freedesktop.portal.Secret",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications retrieve a per-application secret.
/// The secret can then be used for encrypting confidential data inside the sandbox.
trait Secret {
    /// RetrieveSecret method
    fn retrieve_secret(&self, fd: RawFd, options: HashMap<&str, Value>) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

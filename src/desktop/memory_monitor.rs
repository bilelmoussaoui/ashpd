use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.MemoryMonitor",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait MemoryMonitor {
    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

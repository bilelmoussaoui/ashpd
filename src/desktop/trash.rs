use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Trash",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Trash {
    /// TrashFile method
    fn trash_file(&self, fd: std::os::unix::io::RawFd) -> Result<u32>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

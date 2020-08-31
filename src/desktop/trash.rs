use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Trash",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications send files to the trashcan.
trait Trash {
    /// Sends a file to the trashcan.
    /// Applications are allowed to trash a file if they can open it in r/w mode.
    ///
    /// Returns 0 if trashing failed, 1 if trashing succeeded, other values may be returned in the future
    /// FIXME: replace output with an enum
    ///
    /// # Arguments
    ///
    /// * `fd` - the file descriptor
    fn trash_file(&self, fd: RawFd) -> Result<u32>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

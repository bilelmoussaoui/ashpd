use std::collections::HashMap;
use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Value;

#[dbus_proxy(
    interface = "org.freedesktop.portal.Camera",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications access camera devices, such as web cams.
trait Camera {
    /// Requests an access to the camera.
    ///
    /// # Arguments
    ///
    /// * `options` - A HashMap
    ///     * `handle_token` - A string that will be used as the last element of the handle.
    fn access_camera(&self, options: HashMap<&str, Value>) -> Result<String>;

    /// Open a file descriptor to the PipeWire remote where the camera nodes are available.
    /// The file descriptor should be used to create a pw_remote object,
    /// by using pw_remote_connect_fd.
    ///
    /// Returns a File descriptor of an open PipeWire remote.
    fn open_pipe_wire_remote(&self, options: HashMap<&str, Value>) -> Result<RawFd>;

    /// A boolean stating whether there is any cameras available.
    #[dbus_proxy(property)]
    fn is_camera_present(&self) -> Result<bool>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

use std::collections::HashMap;
use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Value;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
#[zvariant(deny_unknown_fields)]
pub struct CameraAccessOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
}

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
    /// * `options` - A [`CameraAccessOptions`]
    ///
    /// [`CameraAccessOptions`]: ./struct.CameraAccessOptions.html
    fn access_camera(&self, options: CameraAccessOptions) -> Result<String>;

    /// Open a file descriptor to the PipeWire remote where the camera nodes are available.
    ///
    /// Returns a File descriptor of an open PipeWire remote.
    ///
    /// # Arguments
    ///
    /// * `options` - ?
    fn open_pipe_wire_remote(&self, options: HashMap<&str, Value>) -> Result<RawFd>;

    /// A boolean stating whether there is any cameras available.
    #[dbus_proxy(property)]
    fn is_camera_present(&self) -> Result<bool>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

use std::collections::HashMap;
use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{OwnedObjectPath, Value};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a camera access request.
pub struct CameraAccessOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
}

#[derive(Debug, Default)]
pub struct CameraAccessOptionsBuilder {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
}

impl CameraAccessOptionsBuilder {
    pub fn handle_token(mut self, handle_token: &str) -> Self {
        self.handle_token = Some(handle_token.to_string());
        self
    }

    pub fn build(self) -> CameraAccessOptions {
        CameraAccessOptions {
            handle_token: self.handle_token,
        }
    }
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
    fn access_camera(&self, options: CameraAccessOptions) -> Result<OwnedObjectPath>;

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

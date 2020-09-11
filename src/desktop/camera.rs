//! # Examples
//!
//! ```no_run
//! use libportal::desktop::camera::{CameraProxy, CameraAccessOptions, AccessCameraResponse};
//! use libportal::RequestProxy;
//!
//! fn main() -> zbus::fdo::Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = CameraProxy::new(&connection)?;
//!
//!     println!("{}", proxy.is_camera_present()?);
//!
//!     let request_handle = proxy.access_camera(CameraAccessOptions::default())?;
//!
//!     let request = RequestProxy::new(&connection, &request_handle)?;
//!     request.on_response(move |response: AccessCameraResponse| {
//!         if response.is_success() {
//!             //let options: HashMap<&str, zvariant::Value> = HashMap::new();
//!             //FIXME: update this once we know which kind of options it takes
//!             //let req = proxy.open_pipe_wire_remote(options).unwrap();
//!             //println!("{:#?}", req);
//!         }
//!     })?;
//!     Ok(())
//! }
//! ```
use crate::ResponseType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{OwnedObjectPath, OwnedValue, Value};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a camera access request.
pub struct CameraAccessOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
}

impl CameraAccessOptions {
    pub fn handle_token(mut self, handle_token: &str) -> Self {
        self.handle_token = Some(handle_token.to_string());
        self
    }
}

#[derive(Debug, Type, Deserialize, Serialize)]
pub struct AccessCameraResponse(ResponseType, HashMap<String, OwnedValue>);

impl AccessCameraResponse {
    pub fn is_success(&self) -> bool {
        self.0 == ResponseType::Success
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
    /// Returns a [`RequestProxy`] handle.
    ///
    /// # Arguments
    ///
    /// * `options` - A [`CameraAccessOptions`]
    ///
    /// [`CameraAccessOptions`]: ./struct.CameraAccessOptions.html
    /// [`RequestProxy`]: ../../request/struct.RequestProxy.html
    fn access_camera(&self, options: CameraAccessOptions) -> Result<OwnedObjectPath>;

    /// Open a file descriptor to the PipeWire remote where the camera nodes are available.
    ///
    /// Returns a File descriptor of an open PipeWire remote.
    ///
    /// # Arguments
    ///
    /// * `options` - ?
    /// FIXME: figure out what are the possible options
    fn open_pipe_wire_remote(&self, options: HashMap<&str, Value>) -> Result<RawFd>;

    /// A boolean stating whether there is any cameras available.
    #[dbus_proxy(property)]
    fn is_camera_present(&self) -> Result<bool>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

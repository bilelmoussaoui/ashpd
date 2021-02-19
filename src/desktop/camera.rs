//! # Examples
//!
//! ```no_run
//! use ashpd::desktop::camera::{CameraProxy, CameraAccessOptions};
//! use ashpd::{BasicResponse as Basic, Response, RequestProxy};
//! use zbus::fdo::Result;
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = CameraProxy::new(&connection)?;
//!
//!     println!("{}", proxy.is_camera_present()?);
//!
//!     let request_handle = proxy.access_camera(CameraAccessOptions::default())?;
//!
//!     let request = RequestProxy::new_for_path(&connection, request_handle.as_str())?;
//!     request.connect_response(move |response: Response<Basic>| {
//!         if response.is_ok() {
//!             //let options: HashMap<&str, zvariant::Value> = HashMap::new();
//!             //FIXME: update this once we know which kind of options it takes
//!             //let req = proxy.open_pipe_wire_remote(options).unwrap();
//!             //println!("{:#?}", req);
//!         }
//!         Ok(())
//!     })?;
//!     Ok(())
//! }
//! ```
use crate::HandleToken;
use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{Fd, OwnedObjectPath, Value};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a `access_camera` request.
pub struct CameraAccessOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<HandleToken>,
}

impl CameraAccessOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
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
    /// Returns a [`RequestProxy`] object path..
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
    fn open_pipe_wire_remote(&self, options: HashMap<&str, Value<'_>>) -> Result<Fd>;

    /// A boolean stating whether there is any cameras available.
    #[dbus_proxy(property)]
    fn is_camera_present(&self) -> Result<bool>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

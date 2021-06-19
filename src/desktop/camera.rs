//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::camera;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let pipewire_fd = camera::stream().await?;
//!     // Use the PipeWire file descriptor with GStreamer for example
//!     Ok(())
//! }
//! ```
use std::collections::HashMap;
use std::os::unix::io;
use std::os::unix::io::AsRawFd;

use zvariant::{Fd, Value};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{
    helpers::{call_basic_response_method, call_method, property},
    Error, HandleToken,
};

#[derive(SerializeDict, DeserializeDict, TypeDict, Clone, Debug, Default)]
/// Specified options for a `access_camera` request.
pub struct CameraAccessOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
}

impl CameraAccessOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }
}

/// The interface lets sandboxed applications access camera devices, such as web
/// cams.
pub struct CameraProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> CameraProxy<'a> {
    pub async fn new(connection: &zbus::azync::Connection) -> Result<CameraProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Camera")
            .path("/org/freedesktop/portal/desktop")?
            .destination("org.freedesktop.portal.Desktop")
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Requests an access to the camera.
    ///
    /// # Arguments
    ///
    /// * `options` - A [`CameraAccessOptions`].
    ///
    /// [`CameraAccessOptions`]: ./struct.CameraAccessOptions.html
    pub async fn access_camera(&self, options: CameraAccessOptions) -> Result<(), Error> {
        call_basic_response_method(&self.0, "AccessCamera", &(options)).await
    }

    /// Open a file descriptor to the PipeWire remote where the camera nodes are
    /// available.
    ///
    /// Returns a File descriptor of an open PipeWire remote.
    ///
    /// # Arguments
    ///
    /// * `options` - ?
    /// FIXME: figure out what are the possible options
    pub async fn open_pipe_wire_remote(
        &self,
        options: HashMap<&str, Value<'_>>,
    ) -> Result<Fd, Error> {
        call_method(&self.0, "OpenPipeWireRemote", &(options)).await
    }

    /// A boolean stating whether there is any cameras available.
    pub async fn is_camera_present(&self) -> Result<bool, Error> {
        property(&self.0, "IsCameraPresent").await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}

/// Request access to the camera and start a stream.
///
/// A wrapper around the [`CameraProxy::access_camera`]
/// and [`CameraProxy::open_pipe_wire_remote`].
pub async fn stream() -> Result<io::RawFd, Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = CameraProxy::new(&connection).await?;
    proxy.access_camera(CameraAccessOptions::default()).await?;

    let remote_fd = proxy.open_pipe_wire_remote(HashMap::new()).await?;
    Ok(remote_fd.as_raw_fd())
}

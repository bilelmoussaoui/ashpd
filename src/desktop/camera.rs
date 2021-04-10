//! # Examples
//!
//! ```rust,no_run
//! use ashpd::{desktop::camera, Response};
//! use zbus::fdo::Result;
//!
//! async fn run() -> Result<()> {
//!     if let Ok(Response::Ok(pipewire_fd)) = camera::stream().await {
//!         // Use the PipeWire file descriptor with GStreamer for example
//!     }
//!     Ok(())
//! }
//! ```
use std::collections::HashMap;
use std::os::unix::io;
use std::os::unix::io::AsRawFd;
use std::sync::Arc;

use futures::{lock::Mutex, FutureExt};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{Fd, Value};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{AsyncRequestProxy, BasicResponse, HandleToken, RequestProxy, Response};

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

#[dbus_proxy(
    interface = "org.freedesktop.portal.Camera",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications access camera devices, such as web
/// cams.
trait Camera {
    /// Requests an access to the camera.
    ///
    /// # Arguments
    ///
    /// * `options` - A [`CameraAccessOptions`].
    ///
    /// [`CameraAccessOptions`]: ./struct.CameraAccessOptions.html
    #[dbus_proxy(object = "Request")]
    fn access_camera(&self, options: CameraAccessOptions);

    /// Open a file descriptor to the PipeWire remote where the camera nodes are
    /// available.
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

    /// The version of this DBus interface.
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

/// Request access to the camera and start a stream.
///
/// An async function around the `AsyncCameraProxy::access_camera`
/// and `AsyncCameraProxy::open_pipe_wire_remote`.
pub async fn stream() -> zbus::Result<Response<io::RawFd>> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncCameraProxy::new(&connection)?;
    let request = proxy.access_camera(CameraAccessOptions::default()).await?;

    let (sender, receiver) = futures::channel::oneshot::channel();
    let sender = Arc::new(Mutex::new(Some(sender)));
    let request_id = request
        .connect_response(move |response: Response<BasicResponse>| {
            let s = sender.clone();
            async move {
                if let Some(m) = s.lock().await.take() {
                    let _ = m.send(response);
                }
                Ok(())
            }
            .boxed()
        })
        .await?;

    while request.next_signal().await?.is_some() {}
    request.disconnect_signal(request_id).await?;

    if let Response::Err(err) = receiver.await.unwrap() {
        return Ok(Response::Err(err));
    }
    let remote_fd = proxy.open_pipe_wire_remote(HashMap::new()).await?;
    Ok(Response::Ok(remote_fd.as_raw_fd()))
}


/// Check whether a camera is present.
///
/// An helper around the `AsyncCameraProxy::is_camera_present` property.
pub async fn is_present() -> zbus::Result<bool> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncCameraProxy::new(&connection)?;
    proxy.is_camera_present().await
}

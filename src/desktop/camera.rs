//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::camera::CameraProxy;
//!
//! pub async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = CameraProxy::new(&connection).await?;
//!     if proxy.is_camera_present().await? {
//!         proxy.access_camera().await?;
//!
//!         let remote_fd = proxy.open_pipe_wire_remote().await?;
//!         // pass the remote fd to GStreamer for example
//!     }
//!     Ok(())
//! }
//! ```

use std::{
    collections::HashMap,
    os::unix::prelude::{IntoRawFd, RawFd},
};

use zbus::zvariant::{DeserializeDict, OwnedFd, SerializeDict, Type, Value};

use super::{HandleToken, DESTINATION, PATH};
use crate::{
    helpers::{call_basic_response_method, call_method},
    Error,
};

#[derive(SerializeDict, DeserializeDict, Type, Clone, Debug, Default)]
/// Specified options for a [`CameraProxy::access_camera`] request.
#[zvariant(signature = "dict")]
struct CameraAccessOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
}

/// The interface lets sandboxed applications access camera devices, such as web
/// cams.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Camera`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Camera).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Camera")]
pub struct CameraProxy<'a>(zbus::Proxy<'a>);

impl<'a> CameraProxy<'a> {
    /// Create a new instance of [`CameraProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<CameraProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Camera")?
            .path(PATH)?
            .destination(DESTINATION)?
            .build()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::Proxy<'_> {
        &self.0
    }

    /// Requests an access to the camera.
    ///
    /// # Specifications
    ///
    /// See also [`AccessCamera`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Camera.AccessCamera).
    #[doc(alias = "AccessCamera")]
    #[doc(alias = "xdp_portal_access_camera")]
    pub async fn access_camera(&self) -> Result<(), Error> {
        let options = CameraAccessOptions::default();
        call_basic_response_method(
            self.inner(),
            &options.handle_token,
            "AccessCamera",
            &(&options),
        )
        .await
    }

    /// Open a file descriptor to the PipeWire remote where the camera nodes are
    /// available.
    ///
    /// # Returns
    ///
    /// File descriptor of an open PipeWire remote.
    ///
    /// # Specifications
    ///
    /// See also [`OpenPipeWireRemote`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Camera.OpenPipeWireRemote).
    #[doc(alias = "OpenPipeWireRemote")]
    #[doc(alias = "xdp_portal_open_pipewire_remote_for_camera")]
    pub async fn open_pipe_wire_remote(&self) -> Result<RawFd, Error> {
        // `options` parameter doesn't seems to be used yet
        // see https://github.com/flatpak/xdg-desktop-portal/blob/master/src/camera.c#L178
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let fd: OwnedFd = call_method(self.inner(), "OpenPipeWireRemote", &(options)).await?;
        Ok(fd.into_raw_fd())
    }

    /// A boolean stating whether there is any cameras available.
    ///
    /// # Specifications
    ///
    /// See also [`IsCameraPresent`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-property-org-freedesktop-portal-Camera.IsCameraPresent).
    #[doc(alias = "IsCameraPresent")]
    #[doc(alias = "xdp_portal_is_camera_present")]
    pub async fn is_camera_present(&self) -> Result<bool, Error> {
        self.inner()
            .get_property::<bool>("IsCameraPresent")
            .await
            .map_err(From::from)
    }
}

/// A helper to get the PipeWire Node ID to use with the camera file descriptor returned by
/// [`CameraProxy::open_pipe_wire_remote`].
///
/// Currently, the camera portal only gives us a file descriptor. Not passing a node id
/// may cause the media session controller to auto-connect the client to an incorrect node.
///
/// The method looks for the available output streams of a `media.role` type of `Camera`
/// and return their Node ID if it found any.
///
/// *Note* The socket referenced by `fd` must not be used while this function is running.
#[cfg(feature = "feature_pipewire")]
pub async fn pipewire_node_id(fd: RawFd) -> Result<Option<u32>, pw::Error> {
    let fd = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 3) };

    if fd == -1 {
        return Err(pw::Error::CreationFailed);
    }

    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = std::sync::Arc::new(std::sync::Mutex::new(Some(sender)));
    std::thread::spawn(move || {
        let inner_sender = sender.clone();
        if let Err(err) = pipewire_node_id_inner(fd, move |node_id| {
            if let Ok(mut guard) = inner_sender.lock() {
                if let Some(inner_sender) = guard.take() {
                    let _result = inner_sender.send(Ok(Some(node_id)));
                }
            }
        }) {
            #[cfg(feature = "log")]
            tracing::error!("Failed to get pipewire node id {:#?}", err);
            let mut guard = sender.lock().unwrap();
            if let Some(sender) = guard.take() {
                let _ = sender.send(Err(err));
            }
        } else {
            #[cfg(feature = "log")]
            tracing::info!("Couldn't find any Node ID");
            let mut guard = sender.lock().unwrap();
            if let Some(sender) = guard.take() {
                let _ = sender.send(Ok(None));
            }
        }
    });
    receiver.await.unwrap()
}

#[cfg(feature = "feature_pipewire")]
fn pipewire_node_id_inner<F: FnOnce(u32) + Clone + 'static>(
    fd: RawFd,
    callback: F,
) -> Result<(), pw::Error> {
    use pw::prelude::*;
    let mainloop = pw::MainLoop::new()?;
    let context = pw::Context::new(&mainloop)?;
    let core = context.connect_fd(fd, None)?;
    let registry = core.get_registry()?;

    let loop_clone = mainloop.clone();
    let _listener_reg = registry
        .add_listener_local()
        .global(move |global| {
            if let Some(props) = &global.props {
                #[cfg(feature = "log")]
                tracing::info!("found properties: {:#?}", props);
                if props.get("media.role") == Some("Camera") {
                    callback.clone()(global.id);
                    loop_clone.quit();
                }
            }
        })
        .register();
    mainloop.run();
    Ok(())
}

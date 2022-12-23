//! Check if a camera is available, request access to it and open a PipeWire
//! remote stream.
//!
//! ### Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::camera::Camera;
//!
//! pub async fn run() -> ashpd::Result<()> {
//!     let camera = Camera::new().await?;
//!     if camera.is_present().await? {
//!         camera.request_access().await?;
//!         let remote_fd = camera.open_pipe_wire_remote().await?;
//!         // pass the remote fd to GStreamer for example
//!     }
//!     Ok(())
//! }
//! ```

use std::{
    collections::HashMap,
    os::fd::{FromRawFd, IntoRawFd, OwnedFd},
};

use zbus::zvariant::{self, SerializeDict, Type, Value};

use super::{HandleToken, DESTINATION, PATH};
use crate::{
    helpers::{call_basic_response_method, call_method, session_connection},
    Error,
};

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct CameraAccessOptions {
    handle_token: HandleToken,
}

/// The interface lets sandboxed applications access camera devices, such as web
/// cams.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Camera`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Camera).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Camera")]
pub struct Camera<'a>(zbus::Proxy<'a>);

impl<'a> Camera<'a> {
    /// Create a new instance of [`Camera`].
    pub async fn new() -> Result<Camera<'a>, Error> {
        let connection = session_connection().await?;
        let proxy = zbus::ProxyBuilder::new_bare(&connection)
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
    pub async fn request_access(&self) -> Result<(), Error> {
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
    pub async fn open_pipe_wire_remote(&self) -> Result<OwnedFd, Error> {
        // `options` parameter doesn't seems to be used yet
        // see https://github.com/flatpak/xdg-desktop-portal/blob/master/src/camera.c#L178
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let fd: zvariant::OwnedFd =
            call_method(self.inner(), "OpenPipeWireRemote", &(options)).await?;
        let raw_fd = fd.into_raw_fd();
        unsafe { Ok(OwnedFd::from_raw_fd(raw_fd)) }
    }

    /// A boolean stating whether there is any cameras available.
    ///
    /// # Specifications
    ///
    /// See also [`IsCameraPresent`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-property-org-freedesktop-portal-Camera.IsCameraPresent).
    #[doc(alias = "IsCameraPresent")]
    #[doc(alias = "xdp_portal_is_camera_present")]
    pub async fn is_present(&self) -> Result<bool, Error> {
        self.inner()
            .get_property::<bool>("IsCameraPresent")
            .await
            .map_err(From::from)
    }
}

#[cfg(feature = "pipewire")]
fn foreign_dic_to_map<D: pw::prelude::ReadableDict>(foreign: &D) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (key, value) in foreign.iter() {
        map.insert(key.to_string(), value.to_string());
    }
    map
}

#[cfg(feature = "pipewire")]
/// A PipeWire camera stream returned by [`pipewire_streams`].
#[derive(Debug)]
pub struct Stream {
    node_id: u32,
    properties: HashMap<String, String>,
}

#[cfg(feature = "pipewire")]
impl Stream {
    /// The id of the PipeWire node.
    pub fn node_id(&self) -> u32 {
        self.node_id
    }

    /// The node properties.
    pub fn properties(&self) -> HashMap<String, String> {
        self.properties.clone()
    }
}

#[cfg(feature = "pipewire")]
fn pipewire_streams_inner<F: Fn(Stream) + Clone + 'static, G: FnOnce() + Clone + 'static>(
    fd: OwnedFd,
    callback: F,
    done_callback: G,
) -> Result<(), pw::Error> {
    use pw::prelude::ReadableDict;

    let mainloop = pw::MainLoop::new()?;
    let context = pw::Context::new(&mainloop)?;
    let raw_fd = fd.into_raw_fd();
    let core = context.connect_fd(raw_fd, None)?;
    let registry = core.get_registry()?;

    let pending = core.sync(0).expect("sync failed");

    let loop_clone = mainloop.clone();
    let _listener_reg = registry
        .add_listener_local()
        .global(move |global| {
            if let Some(props) = &global.props {
                if props.get("media.role") == Some("Camera") {
                    #[cfg(feature = "tracing")]
                    tracing::info!("found camera: {:#?}", props);

                    let properties = foreign_dic_to_map(props);
                    let node_id = global.id;

                    let stream = Stream {
                        node_id,
                        properties,
                    };
                    callback.clone()(stream);
                }
            }
        })
        .register();
    let _listener_core = core
        .add_listener_local()
        .done(move |id, seq| {
            if id == pw::PW_ID_CORE && seq == pending {
                loop_clone.quit();
                done_callback.clone()();
            }
        })
        .register();

    mainloop.run();

    Ok(())
}

/// A helper to get a list of PipeWire streams to use with the camera file
/// descriptor returned by [`Camera::open_pipe_wire_remote`].
///
/// Currently, the camera portal only gives us a file descriptor. Not passing a
/// node id may cause the media session controller to auto-connect the client to
/// an incorrect node.
///
/// The method looks for the available output streams of a `media.role` type of
/// `Camera` and return a list of `Stream`s.
///
/// *Note* The socket referenced by `fd` must not be used while this function is
/// running.
#[cfg(feature = "pipewire")]
pub async fn pipewire_streams(fd: OwnedFd) -> Result<Vec<Stream>, pw::Error> {
    let (sender, receiver) = futures_channel::oneshot::channel();
    let (streams_sender, mut streams_receiver) = futures_channel::mpsc::unbounded();

    let sender = std::sync::Arc::new(std::sync::Mutex::new(Some(sender)));
    let streams_sender = std::sync::Arc::new(std::sync::Mutex::new(streams_sender));

    std::thread::spawn(move || {
        let inner_sender = sender.clone();

        if let Err(err) = pipewire_streams_inner(
            fd,
            move |stream| {
                let inner_streams_sender = streams_sender.clone();
                if let Ok(mut sender) = inner_streams_sender.lock() {
                    let _result = sender.start_send(stream);
                };
            },
            move || {
                if let Ok(mut guard) = inner_sender.lock() {
                    if let Some(inner_sender) = guard.take() {
                        let _result = inner_sender.send(Ok(()));
                    }
                }
            },
        ) {
            #[cfg(feature = "tracing")]
            tracing::error!("Failed to get pipewire streams {:#?}", err);
            let mut guard = sender.lock().unwrap();
            if let Some(sender) = guard.take() {
                let _ = sender.send(Err(err));
            }
        }
    });

    receiver.await.unwrap()?;

    let mut streams = vec![];
    while let Ok(Some(stream)) = streams_receiver.try_next() {
        streams.push(stream);
    }

    Ok(streams)
}

#[cfg(not(feature = "pipewire"))]
pub async fn request() -> Result<Option<OwnedFd>, Error> {
    let proxy = Camera::new().await?;
    proxy.request_access().await?;
    if proxy.is_present().await? {
        Ok(Some(proxy.open_pipe_wire_remote().await?))
    } else {
        Ok(None)
    }
}

#[cfg(feature = "pipewire")]
pub async fn request() -> Result<Option<(OwnedFd, Vec<Stream>)>, Error> {
    let proxy = Camera::new().await?;
    proxy.request_access().await?;
    if proxy.is_present().await? {
        let fd = proxy.open_pipe_wire_remote().await?;
        let dup_fd = fd.try_clone().unwrap();
        let streams = pipewire_streams(dup_fd).await?;
        Ok(Some((fd, streams)))
    } else {
        Ok(None)
    }
}

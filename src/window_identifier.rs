use serde::{ser::Serializer, Serialize};
use zbus::zvariant::{Signature, Type};

#[cfg(any(feature = "feature_gtk4", feature = "feature_gtk3"))]
use futures::lock::Mutex;
#[cfg(any(feature = "feature_gtk4", feature = "feature_gtk3"))]
use std::sync::Arc;

// This is needed for docs so that we include glib only once
#[cfg(all(feature = "feature_gtk4", not(feature = "feature_gtk3")))]
use gtk4::glib::{self, clone};
#[cfg(feature = "feature_gtk4")]
use gtk4::prelude::*;

#[cfg(feature = "feature_gtk3")]
use gtk3::{
    glib::{self, clone},
    prelude::*,
};

#[cfg(feature = "raw_handle")]
use raw_window_handle::{RawWindowHandle, WaylandHandle, XlibHandle};
/// Most portals interact with the user by showing dialogs.
/// These dialogs should generally be placed on top of the application window
/// that triggered them. To arrange this, the compositor needs to know about the
/// application window. Many portal requests expect a [`WindowIdentifier`] for
/// this reason.
///
/// Under X11, the [`WindowIdentifier`] should have the form `x11:XID`, where
/// XID is the XID of the application window in hexadecimal. Under Wayland, it should have the
/// form `wayland:HANDLE`, where HANDLE is a surface handle obtained with the
/// [xdg-foreign](https://github.com/wayland-project/wayland-protocols/blob/master/unstable/xdg-foreign/xdg-foreign-unstable-v2.xml) protocol.
///
/// See also [Parent window identifiers](https://flatpak.github.io/xdg-desktop-portal/index.html#parent_window).
///
/// # Usage
///
/// ## With GTK 4
///
/// The feature `feature_gtk4` must be enabled. You can get a
/// [`WindowIdentifier`] from a [`IsA<gtk4::Native>`](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Native.html) using `WindowIdentifier::from_native`
///
/// ```rust, ignore
/// let widget = gtk4::Button::new();
///
/// let ctx = glib::MainContext::default();
/// ctx.spawn_async(async move {
///     let identifier = WindowIdentifier::from_native(&widget.native().unwrap()).await;
///
///     /// Open some portals
/// });
/// ```
/// The constructor should return a valid identifier under both X11 and Wayland
/// and fallback to the [`Default`] implementation otherwise.
///
/// ## With GTK 3
///
/// The feature `feature_gtk3` must be enabled. You can get a
/// [`WindowIdentifier`] from a [`IsA<gdk3::Window>`](https://gtk-rs.org/gtk3-rs/stable/latest/docs/gdk/struct.Window.html) using
/// `WindowIdentifier::from_window`
///
/// ```rust, ignore
/// let widget = gtk4::Button::new();
/// let ctx = glib::MainContext::default();
/// ctx.spawn_async(async move {
///     let identifier = WindowIdentifier::from_window(&widget.window().unwrap()).await;
///
///     /// Open some portals
/// });
/// ```
/// The constructor should return a valid identifier under both X11 and Wayland
/// and fallback to the [`Default`] implementation otherwise.
///
/// ## Other Toolkits
///
/// If you have access to `RawWindowHandle` you can convert it to a [`WindowIdentifier`] with
/// ```rust, ignore
///     let handle = RawWindowHandle::Xlib(XlibHandle::empty());
///     let identifier = WindowIdentifier::from_raw_handle(handle).await;///
/// ```
///
/// In case you don't have access to a WindowIdentifier:
///
/// ```rust
/// use ashpd::WindowIdentifier;
///
/// let identifier = WindowIdentifier::default();
/// ```
/// We would love merge requests that adds other `From<T> for WindowIdentifier`
/// implementations for other toolkits.
pub enum WindowIdentifier {
    /// Gtk 4 Window Identifier
    #[cfg(feature = "feature_gtk4")]
    #[doc(hidden)]
    Gtk4 {
        /// The top level window
        native: Arc<Mutex<Option<gtk4::Native>>>,
        /// The exported window handle
        handle: String,
    },
    /// GTK 3 Window Identifier
    #[cfg(feature = "feature_gtk3")]
    #[doc(hidden)]
    Gtk3 {
        /// The exported window handle
        handle: String,
        // the top level window
        window: Arc<Mutex<Option<gtk3::gdk::Window>>>,
    },
    /// For Other Toolkits
    #[doc(hidden)]
    Other(String),
}

unsafe impl Send for WindowIdentifier {}
unsafe impl Sync for WindowIdentifier {}

impl Type for WindowIdentifier {
    fn signature() -> Signature<'static> {
        String::signature()
    }
}

impl Serialize for WindowIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.inner())
    }
}

impl std::fmt::Display for WindowIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner())
    }
}

impl std::fmt::Debug for WindowIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("WindowIdentifier")
            .field(&self.inner())
            .finish()
    }
}

impl WindowIdentifier {
    /// Create a new window identifier
    pub fn new(identifier: &str) -> Self {
        Self::Other(identifier.to_string())
    }

    pub(crate) fn inner(&self) -> &str {
        match self {
            #[cfg(feature = "feature_gtk4")]
            Self::Gtk4 { native: _, handle } => handle,
            #[cfg(feature = "feature_gtk3")]
            Self::Gtk3 { handle, window: _ } => handle,
            Self::Other(handle) => handle,
        }
    }
}

impl Default for WindowIdentifier {
    fn default() -> Self {
        Self::new("")
    }
}

impl WindowIdentifier {
    #[cfg(feature = "feature_gtk4")]
    /// Creates a [`WindowIdentifier`] from a [`gtk4::Native`](https://docs.gtk.org/gtk4/class.Native.html).
    ///
    /// The constructor returns a valid handle under both Wayland & x11.
    ///
    /// **Note** the function has to be async as the Wayland handle retrieval
    /// API is async as well.
    pub async fn from_native<W: glib::IsA<gtk4::Native>>(native: &W) -> Self {
        let surface = native.surface();
        let backend = surface.display().backend();
        let handle = if backend.is_wayland() {
            let (sender, receiver) = futures::channel::oneshot::channel::<String>();
            let sender = Arc::new(Mutex::new(Some(sender)));

            let top_level = surface
                .downcast_ref::<gdk4wayland::WaylandToplevel>()
                .unwrap();

            top_level.export_handle(clone!(@strong sender => move |_, handle| {
                let wayland_handle = format!("wayland:{}", handle);
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@strong sender, @strong wayland_handle => async move {
                    if let Some(m) = sender.lock().await.take() {
                        let _ = m.send(wayland_handle);
                    }
                }));
            }));
            receiver.await.ok()
        } else if backend.is_x11() {
            surface
                .downcast_ref::<gdk4x11::X11Surface>()
                .map(|w| format!("x11:0x{:x}", w.xid()))
        } else {
            None
        };

        match handle {
            Some(h) => WindowIdentifier::Gtk4 {
                native: Arc::new(Mutex::new(Some(native.clone().upcast()))),
                handle: h,
            },
            None => WindowIdentifier::default(),
        }
    }

    #[cfg(feature = "feature_gtk3")]
    /// Creates a [`WindowIdentifier`] from a [`gdk::Window`](https://developer.gnome.org/gdk3/stable/gdk3-Windows.html).
    ///
    /// The constructor returns a valid handle under both Wayland & x11.
    ///
    /// **Note** the function has to be async as the Wayland handle retrieval
    /// API is async as well.
    pub async fn from_window<W: glib::IsA<gtk3::gdk::Window>>(win: &W) -> Self {
        let backend = win.as_ref().display().backend();
        let handle = if backend.is_wayland() {
            let (sender, receiver) = futures::channel::oneshot::channel::<String>();
            let sender = Arc::new(Mutex::new(Some(sender)));
            let wayland_win = win
                .as_ref()
                .downcast_ref::<gdk3wayland::WaylandWindow>()
                .unwrap();
            wayland_win.export_handle(clone!(@strong sender => move |_, handle| {
                let wayland_handle = format!("wayland:{}", handle);
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@strong sender, @strong wayland_handle => async move {
                    if let Some(m) = sender.lock().await.take() {
                        let _ = m.send(wayland_handle);
                    }
                }));
            }));
            receiver.await.ok()
        } else if backend.is_x11() {
            win.as_ref()
                .downcast_ref::<gdk3x11::X11Window>()
                .map(|w| format!("x11:0x{:x}", w.xid()))
        } else {
            None
        };

        match handle {
            Some(h) => WindowIdentifier::Gtk3 {
                handle: h,
                window: Arc::new(Mutex::new(Some(win.clone().upcast()))),
            },
            None => WindowIdentifier::default(),
        }
    }

    #[cfg(feature = "raw_handle")]
    /// Create an instance of [`WindowIdentifier`] from a [`RawWindowHandle`](raw_window_handle::RawWindowHandle).
    ///
    /// # Panics
    ///
    /// If the passed RawWindowHandle is not of `WaylandHandle` or `XlibHandle` type. As other platforms handles
    /// are not supported by the portals so far.
    pub async fn form_raw_handle(handle: RawWindowHandle) -> Self {
        use raw_window_handle::RawWindowHandle::{Wayland, Xlib};
        match handle {
            // TODO: figure out how to make use of the wl_display to export a handle with xdg-foreign protocol
            Wayland(_wl_handle) => Self::default(),
            Xlib(x_handle) => {
                #[cfg(feature = "feature_gtk3")]
                {
                    Self::Gtk3 {
                        handle: format!("x11:0x{:x}", x_handle.window),
                        window: Default::default(),
                    }
                }
                #[cfg(feature = "feature_gtk4")]
                {
                    Self::Gtk4 {
                        handle: format!("x11:0x{:x}", x_handle.window),
                        native: Default::default(),
                    }
                }
                #[cfg(not(any(feature = "feature_gtk3", feature = "feature_gtk4")))]
                {
                    Self::default()
                }
            }
            _ => Self::default(), // Fallback to default
        }
    }

    #[cfg(feature = "raw_handle")]
    /// Convert a [`WindowIdentifier`] to [`RawWindowHandle`](raw_window_handle::RawWindowHandle`).
    ///
    /// # Panics
    ///
    /// If you attempt to convert a [`WindowIdentifier`] created from a [`RawWindowHandle`](raw_window_handle::RawWindowHandle`)
    /// instead of the gtk3 / gtk4 constructors.
    pub async fn as_raw_handle(&self) -> RawWindowHandle {
        unsafe {
            match self {
                #[cfg(feature = "feature_gtk4")]
                Self::Gtk4 { native, .. } => {
                    use gtk4::gdk::Backend;
                    use gtk4::glib::translate::ToGlibPtr;

                    let native = native
                        .lock()
                        .await
                        .as_ref()
                        .expect("Can't create a RawWindowHandle without a gtk::Native")
                        .clone();
                    let surface = native.surface();
                    let display = surface.display();
                    match display.backend() {
                        Backend::Wayland => {
                            let mut wayland_handle = WaylandHandle::empty();
                            wayland_handle.surface =
                                gdk4wayland::ffi::gdk_wayland_surface_get_wl_surface(
                                    surface
                                        .downcast_ref::<gdk4wayland::WaylandSurface>()
                                        .unwrap()
                                        .to_glib_none()
                                        .0,
                                );
                            wayland_handle.display =
                                gdk4wayland::ffi::gdk_wayland_display_get_wl_display(
                                    display
                                        .downcast_ref::<gdk4wayland::WaylandDisplay>()
                                        .unwrap()
                                        .to_glib_none()
                                        .0,
                                );
                            RawWindowHandle::Wayland(wayland_handle)
                        }
                        Backend::X11 => {
                            let mut x11_handle = XlibHandle::empty();
                            x11_handle.window =
                                surface.downcast_ref::<gdk4x11::X11Surface>().unwrap().xid();
                            x11_handle.display = gdk4x11::ffi::gdk_x11_display_get_xdisplay(
                                display
                                    .downcast_ref::<gdk4x11::X11Display>()
                                    .unwrap()
                                    .to_glib_none()
                                    .0,
                            );
                            RawWindowHandle::Xlib(x11_handle)
                        }
                        _ => unreachable!(),
                    }
                }
                #[cfg(feature = "feature_gtk3")]
                Self::Gtk3 { window, .. } => {
                    use gtk3::gdk::Backend;
                    use gtk3::glib::translate::ToGlibPtr;

                    let window: gtk3::gdk::Window = window
                        .lock()
                        .await
                        .as_ref()
                        .expect("Can't create a RawWindowHandle without a GdkWindow")
                        .clone();
                    let display = window.display();
                    match display.backend() {
                        Backend::Wayland => {
                            let mut wayland_handle = WaylandHandle::empty();
                            wayland_handle.surface =
                                gdk3wayland::ffi::gdk_wayland_window_get_wl_surface(
                                    window
                                        .downcast_ref::<gdk3wayland::WaylandWindow>()
                                        .unwrap()
                                        .to_glib_none()
                                        .0,
                                );
                            wayland_handle.display =
                                gdk3wayland::ffi::gdk_wayland_display_get_wl_display(
                                    display
                                        .downcast_ref::<gdk3wayland::WaylandDisplay>()
                                        .unwrap()
                                        .to_glib_none()
                                        .0,
                                );
                            RawWindowHandle::Wayland(wayland_handle)
                        }
                        Backend::X11 => {
                            let mut x11_handle = XlibHandle::empty();
                            x11_handle.window =
                                window.downcast_ref::<gdk3x11::X11Window>().unwrap().xid();
                            x11_handle.display = gdk3x11::ffi::gdk_x11_display_get_xdisplay(
                                display
                                    .downcast_ref::<gdk3x11::X11Display>()
                                    .unwrap()
                                    .to_glib_none()
                                    .0,
                            ) as *mut _;
                            RawWindowHandle::Xlib(x11_handle)
                        }
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

impl Drop for WindowIdentifier {
    fn drop(&mut self) {
        #[cfg(feature = "feature_gtk4")]
        if let Self::Gtk4 { native, handle: _ } = self {
            let ctx = glib::MainContext::default();
            ctx.spawn_local(clone!(@strong native => async move {
                if let Some(native) = native.lock().await.as_ref() {
                    let surface = native.surface();
                    if surface.display().backend().is_wayland() {
                        let top_level = surface.downcast_ref::<gdk4wayland::WaylandToplevel>().unwrap();
                        top_level.unexport_handle();
                    }
                };
            }));
        }
        #[cfg(feature = "feature_gtk3")]
        if let Self::Gtk3 { window, handle: _ } = self {
            let ctx = glib::MainContext::default();
            ctx.spawn_local(clone!(@strong window => async move {
                if let Some(window) = window.lock().await.as_ref() {
                    if window.display().backend().is_wayland() {
                        let wayland_win = window.downcast_ref::<gdk3wayland::WaylandWindow>().unwrap();
                        wayland_win.unexport_handle();
                    }
                }
            }));
        }
    }
}

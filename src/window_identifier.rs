use serde::{ser::Serializer, Serialize};

#[cfg(any(feature = "feature_gtk4", feature = "feature_gtk3"))]
use futures::lock::Mutex;
#[cfg(any(feature = "feature_gtk4", feature = "feature_gtk3"))]
use std::sync::Arc;

#[cfg(feature = "feature_gtk4")]
use gtk4::{glib, prelude::*};

#[cfg(feature = "feature_gtk3")]
use gtk3::{
    gdk as gdk3,
    glib::{self, translate::*},
    prelude::*,
};
#[cfg(feature = "feature_gtk3")]
use std::{ffi::c_void, os::raw::c_char};

#[derive(Clone)]
/// Most portals interact with the user by showing dialogs.
/// These dialogs should generally be placed on top of the application window
/// that triggered them. To arrange this, the compositor needs to know about the
/// application window. Many portal requests expect a [`WindowIdentifier`] for
/// this reason.
///
/// Under X11, the [`WindowIdentifier`] should have the form `x11:XID`, where
/// XID is the XID of the application window. Under Wayland, it should have the
/// form `wayland:HANDLE`, where HANDLE is a surface handle obtained with the
/// [xdg-foreign](https://github.com/wayland-project/wayland-protocols/blob/master/unstable/xdg-foreign/xdg-foreign-unstable-v2.xml) protocol.
///
/// See also [Parent window identifiers](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html#parent_window).
///
/// # Usage
///
/// ## With GTK 4
///
/// The feature `feature_gtk4` must be enabled. You can get a
/// [`WindowIdentifier`] from a `gtk4::Native` using `WindowIdentifier::from_native`
///
/// ```rust, ignore
/// let widget = gtk4::Button::new();
///
/// let ctx = glib::MainContext::default();
/// ctx.spawn_async(async move {
///     let identifier = WindowIdentifier::from_native(widget.root().unwrap()).await;
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
/// [`WindowIdentifier`] from a `gdk3::Window` using
/// `WindowIdentifier::from_window`
///
/// ```rust, ignore
/// let widget = gtk4::Button::new();
/// let ctx = glib::MainContext::default();
/// ctx.spawn_async(async move {
///     let identifier = WindowIdentifier::from_window(widget.window().unwrap()).await;
///
///     /// Open some portals
/// });
/// ```
/// The constructor should return a valid identifier under both X11 and Wayland
/// and fallback to the [`Default`] implementation otherwise.
///
/// ## Other Toolkits
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
        native: Arc<Mutex<gtk4::Native>>,
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
        window: Arc<Mutex<gdk3::Window>>,
    },
    /// For Other Toolkits
    #[doc(hidden)]
    Other(String),
}

unsafe impl Send for WindowIdentifier {}
unsafe impl Sync for WindowIdentifier {}

impl zvariant::Type for WindowIdentifier {
    fn signature() -> zvariant::Signature<'static> {
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
        let surface = native.surface().unwrap();
        let handle = match surface
            .display()
            .expect("Surface has to be attached to a display")
            .type_()
            .name()
        {
            "GdkWaylandDisplay" => {
                let (sender, receiver) = futures::channel::oneshot::channel::<String>();
                let sender = Arc::new(Mutex::new(Some(sender)));

                let top_level = surface
                    .downcast_ref::<gdk4wayland::WaylandToplevel>()
                    .unwrap();

                top_level.export_handle(glib::clone!(@strong sender => move |_level, handle| {
                    let wayland_handle = format!("wayland:{}", handle);
                    let ctx = glib::MainContext::default();
                    ctx.spawn_local(glib::clone!(@strong sender, @strong wayland_handle => async move {
                        if let Some(m) = sender.lock().await.take() {
                            let _ = m.send(wayland_handle);
                        }
                    }));
                }));
                receiver.await.ok()
            }
            "GdkX11Display" => surface
                .downcast_ref::<gdk4x11::X11Surface>()
                .map(|w| format!("x11:{}", w.xid())),
            _ => None,
        };

        match handle {
            Some(h) => WindowIdentifier::Gtk4 {
                native: Arc::new(Mutex::new(native.clone().upcast())),
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
    pub async fn from_window<W: glib::IsA<gdk3::Window>>(win: &W) -> Self {
        let handle = match win.as_ref().display().type_().name() {
            "GdkWaylandDisplay" => {
                let (sender, receiver) = futures::channel::oneshot::channel::<String>();
                let sender = Arc::new(Mutex::new(Some(sender)));

                export_wayland_handle(
                    win,
                    glib::clone!(@strong sender => move |_level, handle| {
                        let wayland_handle = format!("wayland:{}", handle);
                        let ctx = glib::MainContext::default();
                        ctx.spawn_local(glib::clone!(@strong sender, @strong wayland_handle => async move {
                            if let Some(m) = sender.lock().await.take() {
                                let _ = m.send(wayland_handle);
                            }
                        }));
                    }),
                );
                receiver.await.ok()
            }
            "GdkX11Display" => win
                .as_ref()
                .downcast_ref::<gdk3x11::X11Window>()
                .map(|w| format!("x11:{}", w.xid())),
            _ => None,
        };

        match handle {
            Some(h) => WindowIdentifier::Gtk3 {
                handle: h,
                window: Arc::new(Mutex::new(win.clone().upcast())),
            },
            None => WindowIdentifier::default(),
        }
    }
}

impl Drop for WindowIdentifier {
    fn drop(&mut self) {
        #[cfg(feature = "feature_gtk4")]
        if let Self::Gtk4 { native, handle: _ } = self {
            let ctx = glib::MainContext::default();
            ctx.spawn_local(glib::clone!(@strong native => async move {
                let native = native.lock().await;
                let surface = native.surface().unwrap();
                let name = surface.display()
                .unwrap()
                .type_()
                .name();
                if name == "GdkWaylandDisplay"
            {
                let top_level = surface.downcast_ref::<gdk4wayland::WaylandToplevel>().unwrap();
                top_level.unexport_handle();
            }
            }));
        }
        #[cfg(feature = "feature_gtk3")]
        if let Self::Gtk3 { window, handle: _ } = self {
            let ctx = glib::MainContext::default();
            ctx.spawn_local(glib::clone!(@strong window => async move {
                let window = window.lock().await;
                let name = window.display().type_().name();
                if name == "GdkWaylandDisplay" {
                    unexport_wayland_handle(&*window);
                }
            }));
        }
    }
}

#[cfg(feature = "feature_gtk3")]
pub(crate) fn unexport_wayland_handle<W: glib::IsA<gdk3::Window>>(win: &W) {
    extern "C" {
        pub fn gdk_wayland_window_unexport_handle(window: *mut gdk3::ffi::GdkWindow);
    }
    unsafe {
        gdk_wayland_window_unexport_handle(win.as_ptr() as *mut _);
    }
}

#[cfg(feature = "feature_gtk3")]
pub(crate) fn export_wayland_handle<
    W: glib::IsA<gdk3::Window>,
    P: Fn(&gdk3::Window, &str) + 'static,
>(
    win: &W,
    callback: P,
) -> bool {
    extern "C" {
        pub fn gdk_wayland_window_export_handle(
            window: *mut gdk3::ffi::GdkWindow,
            cb: Option<unsafe extern "C" fn(*mut gdk3::ffi::GdkWindow, *const c_char, *mut c_void)>,
            user_data: *mut c_void,
            destroy_notify: Option<unsafe extern "C" fn(*mut c_void)>,
        ) -> bool;
    }
    unsafe extern "C" fn callback_trampoline<P: Fn(&gdk3::Window, &str) + 'static>(
        window: *mut gdk3::ffi::GdkWindow,
        handle: *const c_char,
        user_data: glib::ffi::gpointer,
    ) {
        let window = from_glib_borrow(window);
        let handle: Borrowed<glib::GString> = from_glib_borrow(handle);
        let callback: &P = &*(user_data as *mut _);
        (*callback)(&window, handle.as_str());
    }
    unsafe extern "C" fn destroy_notify<P: Fn(&gdk3::Window, &str) + 'static>(
        data: glib::ffi::gpointer,
    ) {
        Box::from_raw(data as *mut _);
    }
    unsafe {
        gdk_wayland_window_export_handle(
            win.as_ref().to_glib_none().0,
            Some(callback_trampoline::<P> as _),
            Box::into_raw(Box::new(callback)) as *mut _,
            Some(destroy_notify::<P> as _),
        )
    }
}

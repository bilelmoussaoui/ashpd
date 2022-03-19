use serde::{ser::Serializer, Serialize};
use zbus::zvariant::{Signature, Type};
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
/// ## From an X11 XID
///
/// ```rust,ignore
/// let identifier = WindowIdentifier::from_xid(212321);
/// ```
///
/// ## From a Wayland Surface
///
/// The feature `wayland` must be enabled.
///
/// ```text
/// // let wl_surface = some_surface;
/// // let identifier = WindowIdentifier::from_wayland(wl_surface as *mut _);
/// ```
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
///     let identifier = WindowIdentifier::from_raw_handle(handle);///
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
#[doc(alias = "XdpParent")]
pub enum WindowIdentifier {
    /// Gtk 4 Window Identifier
    #[cfg(feature = "feature_gtk4")]
    #[doc(hidden)]
    Gtk4(Gtk4WindowIdentifier),
    /// GTK 3 Window Identifier
    #[cfg(feature = "feature_gtk3")]
    #[doc(hidden)]
    Gtk3(Gtk3WindowIdentifier),
    #[cfg(feature = "wayland")]
    #[doc(hidden)]
    Wayland(WaylandWindowIdentifier),
    #[doc(hidden)]
    X11(X11WindowIdentifier),
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
        serializer.serialize_str(&self.inner())
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

impl Default for WindowIdentifier {
    fn default() -> Self {
        Self::new("")
    }
}

impl WindowIdentifier {
    /// Create a new window identifier
    pub fn new(identifier: &str) -> Self {
        Self::Other(identifier.to_string())
    }

    pub(crate) fn inner(&self) -> String {
        match self {
            #[cfg(feature = "feature_gtk4")]
            Self::Gtk4(identifier) => identifier.to_string(),
            #[cfg(feature = "feature_gtk3")]
            Self::Gtk3(identifier) => identifier.to_string(),
            #[cfg(feature = "wayland")]
            Self::Wayland(identifier) => identifier.to_string(),
            Self::X11(identifier) => identifier.to_string(),
            Self::Other(handle) => handle.to_owned(),
        }
    }

    #[cfg(feature = "feature_gtk4")]
    /// Creates a [`WindowIdentifier`] from a [`gtk4::Native`](https://docs.gtk.org/gtk4/class.Native.html).
    ///
    /// The constructor returns a valid handle under both Wayland & x11.
    ///
    /// **Note** the function has to be async as the Wayland handle retrieval
    /// API is async as well.
    #[doc(alias = "xdp_parent_new_gtk")]
    pub async fn from_native(native: &impl ::gtk4::glib::IsA<::gtk4::Native>) -> Self {
        match Gtk4WindowIdentifier::new(native).await {
            Some(identifier) => Self::Gtk4(identifier),
            None => Self::default(),
        }
    }

    #[cfg(feature = "feature_gtk3")]
    #[doc(alias = "xdp_parent_new_gtk")]
    /// Creates a [`WindowIdentifier`] from a [`gdk::Window`](https://developer.gnome.org/gdk3/stable/gdk3-Windows.html).
    ///
    /// The constructor returns a valid handle under both Wayland & x11.
    ///
    /// **Note** the function has to be async as the Wayland handle retrieval
    /// API is async as well.
    pub async fn from_window(win: &impl ::gtk3::glib::IsA<::gtk3::gdk::Window>) -> Self {
        match Gtk3WindowIdentifier::new(win).await {
            Some(identifier) => Self::Gtk3(identifier),
            None => Self::default(),
        }
    }

    #[cfg(feature = "raw_handle")]
    /// Create an instance of [`WindowIdentifier`] from a [`RawWindowHandle`](raw_window_handle::RawWindowHandle).
    ///
    /// The constructor returns a valid handle under both Wayland & X11.
    pub fn from_raw_handle(handle: &raw_window_handle::RawWindowHandle) -> Self {
        use raw_window_handle::RawWindowHandle::{Wayland, Xcb, Xlib};
        match handle {
            Wayland(wl_handle) => unsafe { Self::from_wayland_raw(wl_handle.surface) },
            Xlib(x_handle) => Self::from_xid(x_handle.window),
            Xcb(x_handle) => Self::from_xid(x_handle.window.into()),
            _ => Self::default(), // Fallback to default
        }
    }

    /// Create an instance of [`WindowIdentifier`] from an X11 window's XID.
    pub fn from_xid(xid: u64) -> Self {
        Self::X11(X11WindowIdentifier::new(xid))
    }

    #[cfg(feature = "wayland")]
    /// Create an instance of [`WindowIdentifier`] from a Wayland surface.
    ///
    /// ## Safety
    ///
    /// The surface has to be a valid Wayland surface ptr
    pub unsafe fn from_wayland_raw(surface_ptr: *mut std::ffi::c_void) -> Self {
        match WaylandWindowIdentifier::from_raw(surface_ptr) {
            Some(identifier) => Self::Wayland(identifier),
            None => Self::default(),
        }
    }

    #[cfg(feature = "wayland")]
    /// Create an instance of [`WindowIdentifier`] from a Wayland surface.
    pub fn from_wayland(surface: &wayland_client::protocol::wl_surface::WlSurface) -> Self {
        match WaylandWindowIdentifier::new(surface) {
            Some(identifier) => Self::Wayland(identifier),
            None => Self::default(),
        }
    }

    #[cfg(all(
        feature = "raw_handle",
        any(feature = "feature_gtk3", feature = "feature_gtk4")
    ))]
    /// Convert a [`WindowIdentifier`] to [`RawWindowHandle`](raw_window_handle::RawWindowHandle`).
    ///
    /// # Panics
    ///
    /// If you attempt to convert a [`WindowIdentifier`] created from a [`RawWindowHandle`](raw_window_handle::RawWindowHandle`)
    /// instead of the gtk3 / gtk4 constructors.
    pub fn as_raw_handle(&self) -> raw_window_handle::RawWindowHandle {
        match self {
            #[cfg(feature = "feature_gtk4")]
            Self::Gtk4(identifier) => identifier.as_raw_handle(),
            #[cfg(feature = "feature_gtk3")]
            Self::Gtk3(identifier) => identifier.as_raw_handle(),
            _ => unreachable!(),
        }
    }
}

/// Supported WindowIdentifier kinds
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WindowType {
    X11(u64),
    Wayland(String),
}

#[cfg(feature = "feature_gtk4")]
mod gtk4;

#[cfg(feature = "feature_gtk4")]
pub use self::gtk4::Gtk4WindowIdentifier;

#[cfg(feature = "feature_gtk3")]
mod gtk3;

#[cfg(feature = "feature_gtk3")]
pub use self::gtk3::Gtk3WindowIdentifier;

#[cfg(any(
    feature = "wayland",
    feature = "feature_gtk3",
    feature = "feature_gtk4"
))]
mod wayland;

#[cfg(feature = "wayland")]
pub use self::wayland::WaylandWindowIdentifier;

mod x11;

pub use self::x11::X11WindowIdentifier;

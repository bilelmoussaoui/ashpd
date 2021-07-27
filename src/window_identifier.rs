use serde::{ser::Serializer, Serialize};

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
/// The feature `feature_gtk4` must be enabled. You can get a [`WindowIdentifier`] from a `gtk4::Root` using `WindowIdentifier::from_root`
///
/// ```rust, ignore
/// let widget = gtk::Button::new();
///
/// let ctx = glib::MainContext::default();
/// ctx.spawn_async(async move {
///     let identifier = WindowIdentifier::from_root(widget.root().unwrap()).await;
///
///     /// Open some portals
/// });
/// ```
/// The constructor should return a valid identifier under both X11 and Wayland and fallback to the [`Default`] implementation otherwise.
///
/// ## With GTK 3
///
/// The feature `feature_gtk3` must be enabled. You can get a [`WindowIdentifier`] from a `gdk3::Window` using `WindowIdentifier::from_window`
///
/// ```rust, ignore
/// let widget = gtk::Button::new();
/// let ctx = glib::MainContext::default();
/// ctx.spawn_async(async move {
///     let identifier = WindowIdentifier::from_window(widget.window().unwrap()).await;
///
///     /// Open some portals
/// });
/// ```
/// The constructor should return a valid identifier under both X11 and Wayland and fallback to the [`Default`] implementation otherwise.
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
        root: gtk4::Root,
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
        window: gtk3::gdk::Window,
    },
    /// For Other Toolkits
    #[doc(hidden)]
    Other(String),
}

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
            Self::Gtk4 { root: _, handle } => handle,
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
    /// Creates a [`WindowIdentifier`] from a [`gtk::Root`](https://gnome.pages.gitlab.gnome.org/gtk/gtk4/iface.Root.html).
    /// `gtk::Root` is the interface implemented by all the widgets that can act
    /// as a top level widget.
    ///
    /// The constructor returns a valid handle under both Wayland & x11.
    ///
    /// **Note** the function has to be async as the Wayland handle retrieval
    /// API is async as well.
    pub async fn from_root<W: gtk4::glib::IsA<gtk4::Root>>(win: &W) -> Self {
        use futures::lock::Mutex;
        use gtk4::glib;
        use gtk4::prelude::{Cast, NativeExt, ObjectExt, SurfaceExt};
        use std::sync::Arc;

        let surface = win
            .as_ref()
            .surface()
            .expect("The window has to be mapped first");

        let handle = match surface
            .display()
            .expect("Surface has to be attached to a display")
            .type_()
            .name()
            .as_ref()
        {
            "GdkWaylandDisplay" => {
                let (sender, receiver) = futures::channel::oneshot::channel::<String>();
                let sender = Arc::new(Mutex::new(Some(sender)));

                let top_level = surface.downcast::<gdk4wayland::WaylandToplevel>().unwrap();

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
            "GdkX11Display" => match surface.downcast::<gdk4x11::X11Surface>().map(|w| w.xid()) {
                Ok(xid) => Some(format!("x11:{}", xid)),
                Err(_) => None,
            },
            _ => None,
        };

        match handle {
            Some(h) => WindowIdentifier::Gtk4 {
                root: win.clone().upcast(),
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
    pub async fn from_window<W: gtk3::glib::IsA<gtk3::gdk::Window>>(win: &W) -> Self {
        use gtk3::prelude::{Cast, ObjectExt};

        let handle = match win.as_ref().display().type_().name().as_ref() {
            "GdkWaylandDisplay" => {
                use futures::lock::Mutex;
                use gtk3::glib;
                use std::sync::Arc;
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
                .map(|w| w.xid())
                .map(|xid| format!("x11:{}", xid)),
            _ => None,
        };

        match handle {
            Some(h) => WindowIdentifier::Gtk3 {
                handle: h,
                window: win.clone().upcast(),
            },
            None => WindowIdentifier::default(),
        }
    }
}

impl Drop for WindowIdentifier {
    fn drop(&mut self) {
        #[cfg(feature = "feature_gtk4")]
        if let Self::Gtk4 { root, handle: _ } = self {
            use gtk4::prelude::{Cast, NativeExt, ObjectExt, SurfaceExt};

            let surface = root.surface().expect("The window has to be mapped first");
            if surface
                .display()
                .expect("Surface has to be attached to a display")
                .type_()
                .name()
                == "GdkWaylandDisplay"
            {
                let top_level = surface.downcast::<gdk4wayland::WaylandToplevel>().unwrap();
                top_level.unexport_handle();
            }
        }
        #[cfg(feature = "feature_gtk3")]
        if let Self::Gtk3 { window, handle: _ } = self {
            use gtk3::prelude::ObjectExt;

            if window.display().type_().name() == "GdkWaylandDisplay" {
                unexport_wayland_handle(window);
            }
        }
    }
}

#[cfg(feature = "feature_gtk3")]
pub(crate) fn unexport_wayland_handle<W: gtk3::glib::IsA<gtk3::gdk::Window>>(win: &W) {
    use gtk3::gdk;

    extern "C" {
        pub fn gdk_wayland_window_unexport_handle(window: *mut gdk::ffi::GdkWindow);
    }
    unsafe {
        gdk_wayland_window_unexport_handle(win.as_ptr() as *mut _);
    }
}

#[cfg(feature = "feature_gtk3")]
pub(crate) fn export_wayland_handle<
    W: gtk3::glib::IsA<gtk3::gdk::Window>,
    P: Fn(&gtk3::gdk::Window, &str) + 'static,
>(
    win: &W,
    callback: P,
) -> bool {
    use gtk3::glib::{self, translate::*};
    use std::ffi::c_void;
    use std::os::raw::c_char;
    extern "C" {
        pub fn gdk_wayland_window_export_handle(
            window: *mut gtk3::gdk::ffi::GdkWindow,
            cb: Option<
                unsafe extern "C" fn(*mut gtk3::gdk::ffi::GdkWindow, *const c_char, *mut c_void),
            >,
            user_data: *mut c_void,
            destroy_notify: Option<unsafe extern "C" fn(*mut c_void)>,
        ) -> bool;
    }
    let callback_data: Box<P> = Box::new(callback);
    unsafe extern "C" fn callback_func<P: Fn(&gtk3::gdk::Window, &str) + 'static>(
        window: *mut gtk3::gdk::ffi::GdkWindow,
        handle: *const c_char,
        user_data: glib::ffi::gpointer,
    ) {
        let window = from_glib_borrow(window);
        let handle: Borrowed<glib::GString> = from_glib_borrow(handle);
        let callback: &P = &*(user_data as *mut _);
        (*callback)(&window, handle.as_str());
    }
    let callback = Some(callback_func::<P> as _);
    unsafe extern "C" fn destroy_notify<P: Fn(&gtk3::gdk::Window, &str) + 'static>(
        data: glib::ffi::gpointer,
    ) {
        let _callback: Box<P> = Box::from_raw(data as *mut _);
    }
    let super_callback0: Box<P> = callback_data;
    unsafe {
        gdk_wayland_window_export_handle(
            win.as_ref().to_glib_none().0,
            callback,
            Box::into_raw(super_callback0) as *mut _,
            Some(destroy_notify::<P> as _),
        )
    }
}

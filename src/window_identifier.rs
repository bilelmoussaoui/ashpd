use serde::{ser::Serializer, Serialize};

#[derive(Clone, Debug)]
/// Most portals interact with the user by showing dialogs.
/// These dialogs should generally be placed on top of the application window
/// that triggered them. To arrange this, the compositor needs to know about the
/// application window. Many portal requests expect a [`WindowIdentifier`] for
/// this reason.
///
/// Under X11, the [`WindowIdentifier`] should have the form `x11:XID`, where
/// XID is the XID of the application window. Under Wayland, it should have the
/// form `wayland:HANDLE`, where HANDLE is a surface handle obtained with the
/// xdg-foreign protocol.
///
/// For other windowing systems, or if you don't have a suitable handle, just
/// use the [`Default`] implementation.
///
/// Please **note** that the `From<gtk3::Window>` implementation is x11 only for
/// now.
///
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
        let handle = match self {
            #[cfg(feature = "feature_gtk4")]
            Self::Gtk4 { root: _, handle } => handle,
            #[cfg(feature = "feature_gtk3")]
            Self::Gtk3 { handle } => handle,
            Self::Other(handle) => handle,
        };
        serializer.serialize_str(handle)
    }
}

impl WindowIdentifier {
    /// Create a new window identifier
    pub fn new(identifier: &str) -> Self {
        Self::Other(identifier.to_string())
    }
}

impl Default for WindowIdentifier {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(feature = "feature_gtk3")]
impl From<gtk3::Window> for WindowIdentifier {
    fn from(win: gtk3::Window) -> Self {
        use gtk3::prelude::{Cast, ObjectExt, WidgetExt};

        let window = win.window().expect("The window has to be mapped first.");

        let handle = match window.display().type_().name().as_ref() {
            /*
            TODO: implement the get_wayland handle
            "GdkWaylandDisplay" => {
                let handle = get_wayland_handle(win).unwrap();
                WindowIdentifier(format!("wayland:{}", handle))
            }*/
            "GdkX11Display" => match window.downcast::<gdk3x11::X11Window>().map(|w| w.xid()) {
                Ok(xid) => Some(format!("x11:{}", xid)),
                Err(_) => None,
            },
            _ => None,
        };

        match handle {
            Some(h) => WindowIdentifier::Gtk3 { handle: h },
            None => WindowIdentifier::default(),
        }
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
    /// **Note** The function has to be async as the Wayland handle retrieval
    /// API is async as well.
    pub async fn from_window<W: gtk4::glib::IsA<gtk4::Root>>(win: &W) -> Self {
        use std::sync::Arc;

        use futures::lock::Mutex;
        use gtk4::glib;
        use gtk4::prelude::{Cast, NativeExt, ObjectExt, SurfaceExt};

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
                root: win.as_ref().clone(),
                handle: h,
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
    }
}

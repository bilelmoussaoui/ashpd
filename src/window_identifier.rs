use serde::{Deserialize, Serialize};
use zvariant_derive::Type;

#[derive(Type, Clone, Debug, Serialize, Deserialize)]
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
/// use the `Default` implementation.
///
/// Please **note** that the `From<gtk3::Window>` and `From<gtk4::Window>`
/// implementation are x11 only for now.
///
/// We would love merge requests that adds other `From<T> for WindowIdentifier`
/// implementations for other toolkits.
///
/// [`WindowIdentifier`]: ./struct.WindowIdentifier.html
pub struct WindowIdentifier(String);

impl WindowIdentifier {
    /// Create a new window identifier
    pub fn new(identifier: &str) -> Self {
        Self(identifier.to_string())
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

        let window = win
            .get_window()
            .expect("The window has to be mapped first.");

        let handle = match window.get_display().get_type().name().as_ref() {
            /*
            TODO: implement the get_wayland handle
            "GdkWaylandDisplay" => {
                let handle = get_wayland_handle(win).unwrap();
                WindowIdentifier(format!("wayland:{}", handle))
            }*/
            "GdkX11Display" => match window.downcast::<gdk3x11::X11Window>().map(|w| w.get_xid()) {
                Ok(xid) => Some(format!("x11:{}", xid)),
                Err(_) => None,
            },
            _ => None,
        };

        match handle {
            Some(h) => WindowIdentifier(h),
            None => WindowIdentifier::default(),
        }
    }
}

impl WindowIdentifier {
    #[cfg(feature = "feature_gtk4")]
    /// Creates a `WindowIdentifier` from a [`gtk::Root`](https://gnome.pages.gitlab.gnome.org/gtk/gtk4/iface.Root.html).
    /// `gtk::Root` is the interface implemented by all the widgets that can act as a top level widget.
    ///
    /// The constructor returns a valid handle under both Wayland & x11.
    ///
    /// **Note** The function has to be async as the Wayland handle retrieval
    /// API is async as well.
    pub async fn from_window<W: gtk4::glib::IsA<gtk4::Root>>(win: &W) -> Self {
        use std::sync::Arc;

        use futures::lock::Mutex;
        use gtk4::glib;
        use gtk4::prelude::{Cast, NativeExt, ObjectExt};

        let surface = win
            .as_ref()
            .get_surface()
            .expect("The window has to be mapped first");

        let handle = match surface
            .get_display()
            .expect("Surface has to be attached to a display")
            .get_type()
            .name()
            .as_ref()
        {
            "GdkWaylandDisplay" => {
                /*
                As the wayland api is async, let's wait till zbus is async ready before
                we do enable it.
                Note: we need to un-export the handle once it's not used anymore automatically
                        using level.unexport_handle();*/

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
            "GdkX11Display" => match surface
                .downcast::<gdk4x11::X11Surface>()
                .map(|w| w.get_xid())
            {
                Ok(xid) => Some(format!("x11:{}", xid)),
                Err(_) => None,
            },
            _ => None,
        };

        match handle {
            Some(h) => WindowIdentifier(h),
            None => WindowIdentifier::default(),
        }
    }
}

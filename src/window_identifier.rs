use serde::{Deserialize, Serialize};
use zvariant_derive::Type;

#[derive(Type, Clone, Debug, Serialize, Deserialize)]
/// Most portals interact with the user by showing dialogs.
/// These dialogs should generally be placed on top of the application window that triggered them.
/// To arrange this, the compositor needs to know about the application window.
/// Many portal requests expect a [`WindowIdentifier`] for this reason.
///
/// Under X11, the [`WindowIdentifier`] should have the form `x11:XID`, where XID is the XID of the application window.
/// Under Wayland, it should have the form `wayland:HANDLE`, where HANDLE is a surface handle obtained with the xdg-foreign protocol.
///
/// For other windowing systems, or if you don't have a suitable handle, just use the `Default` implementation.
///
/// Please note that the `From<gtk3::Window>` and `From<gtk4::Window>` implementation are x11 only for now.
///
/// We would love merge requests that adds other `From<T> for WindowIdentifier` implementations for other toolkits.
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

#[cfg(feature = "feature_gtk4")]
impl From<gtk4::Window> for WindowIdentifier {
    fn from(win: gtk4::Window) -> Self {
        use gtk4::prelude::{Cast, NativeExt, ObjectExt};

        let surface = win
            .get_surface()
            .expect("The window has to be mapped first.");

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
                        using level.unexport_handle();
                let top_level = surface.downcast::<gdk4wayland::WaylandTopLevel>().unwrap();
                top_level.export_handle(move |level, handle| {
                    Some(format!("wayland:{}", handle))
                });*/
                None
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

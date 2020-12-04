use serde::{Deserialize, Serialize};
use zvariant_derive::Type;

#[derive(Type, Clone, Debug, Serialize, Deserialize)]
/// Most portals interact with the user by showing dialogs.
/// These dialogs should generally be placed on top of the application window that triggered them.
/// To arrange this, the compositor needs to know about the application window.
/// Many portal requests expect a [`WindowIdentifier`] for this reason.
///
/// Under X11, the [`WindowIdentifier`] should have the form "x11:XID", where XID is the XID of the application window.
/// Under Wayland, it should have the form "wayland:HANDLE", where HANDLE is a surface handle obtained with the xdg_foreign protocol.
///
/// For other windowing systems, or if you don't have a suitable handle, just use the `Default` implementation.
///
/// Please note that the `From<gtk::Window>` implementation is x11 only for now.
///
/// We would love merge requests that adds other `From<T> for WindowIdentifier` implementations for other toolkits.
///
/// [`WindowIdentifier`]: ./struct.WindowIdentifier.html
///
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
        use gdk3::prelude::{Cast, ObjectExt, WindowExt};
        use gtk3::WidgetExt;

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

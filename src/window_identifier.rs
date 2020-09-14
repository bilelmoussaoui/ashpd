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
/// Normally, we should provide a `From<gtk::Window> for WindowIdentifier` implementation.
/// But as that's currently impossible to do from Rust in a sane way, we should try to provide a C function
/// that gives us a handle from the `Gdk::Window` and call it from Rust in the `From` implementation.
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

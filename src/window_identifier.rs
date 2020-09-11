use serde::{Deserialize, Serialize};
use zvariant_derive::Type;

#[derive(Type, Clone, Debug, Serialize, Deserialize)]
pub struct WindowIdentifier(String);

impl WindowIdentifier {
    pub fn new(identifier: &str) -> Self {
        Self(identifier.to_string())
    }
}

impl Default for WindowIdentifier {
    fn default() -> Self {
        Self::new("")
    }
}

// FIXME: add a GTK feature and
// implement From<gtk::Window> for WindowIdentifier
// it's going to ugly. As of today, there's no nice way to get the window
// handle. You need to check if the window is running under x11
// and then use gdkx11 provides the necessary x11 types to get a handle.
// The same thing should be done for wayland, except it's more complex
// the bindings are not possible currently as the C part uses some C types
// that are not part of any bindings nor havbe bindings support.
// which makes generating the bindings of only the part we need harder
// as we need to fix gtk first, wait for a release, generate the bindings and so on.
//
// The alternative would be to have a C file with a function that gives us a handle
// from a gtk_sys::Window, and call it inside the from implementation

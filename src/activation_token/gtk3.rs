use gtk3::prelude::*;
use gtk3::{gdk, glib};

#[derive(Debug)]
pub struct Gtk3ActivationToken {
    pub(crate) token: String,
}

impl Gtk3ActivationToken {
    pub fn from_window(window: &impl glib::IsA<gdk::Window>) -> Option<Self> {
        let display = window.as_ref().display();
        match display.backend() {
            gdk::Backend::Wayland => todo!(),
            _ => None,
        }
    }
}

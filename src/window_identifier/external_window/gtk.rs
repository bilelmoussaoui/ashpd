#[cfg(feature = "backend_gtk3")]
use gdk3wayland as gdk_wayland;
#[cfg(feature = "backend_gtk3")]
use gdk3x11 as gdk_x11;
#[cfg(feature = "backend_gtk4")]
use gdk4wayland as gdk_wayland;
#[cfg(feature = "backend_gtk4")]
use gdk4x11 as gdk_x11;
#[cfg(feature = "backend_gtk3")]
use gtk3::{self as gtk, gdk, glib, prelude::*};
#[cfg(feature = "backend_gtk4")]
use gtk4::{self as gtk, gdk, glib, prelude::*};

use super::{gtk_wayland::ExternalWaylandWindow, gtk_x11::ExternalX11Window};
use crate::WindowIdentifierType;

/// Helper to convert a [`WindowIdentifierType`] to a GTK Window
/// to pass to `set_transient_for` dialog calls.
pub enum GtkExternalWindow {
    /// Wayland external window.
    Wayland(ExternalWaylandWindow),
    /// X11 external window.
    X11(ExternalX11Window),
}

impl GtkExternalWindow {
    /// Create a new instance of [ ExternalWindow`]
    pub fn new(window_identifier: WindowIdentifierType) -> Option<Self> {
        match window_identifier {
            WindowIdentifierType::Wayland(exported_handle) => {
                ExternalWaylandWindow::new(exported_handle).map(Self::Wayland)
            }
            WindowIdentifierType::X11(foreign_xid) => {
                ExternalX11Window::new(foreign_xid).map(Self::X11)
            }
        }
    }

    /// Mark the external window as parent of a surface.
    pub fn set_parent_of<S: glib::IsA<gdk::Surface>>(&self, surface: &S) {
        match self {
            Self::X11(x11_window) => x11_window.set_parent_of(
                surface
                    .as_ref()
                    .downcast_ref::<gdk_x11::X11Surface>()
                    .unwrap(),
            ),
            Self::Wayland(wl_window) => wl_window.set_parent_of(
                surface
                    .as_ref()
                    .downcast_ref::<gdk_wayland::WaylandSurface>()
                    .unwrap(),
            ),
        }
    }

    /// Create the fake `gtk::Window`
    pub fn fake(maybe_self: Option<&Self>) -> gtk::Window {
        match maybe_self {
            Some(Self::X11(x11_window)) => glib::Object::builder()
                .property("display", x11_window.display())
                .build(),
            Some(Self::Wayland(wl_window)) => glib::Object::builder()
                .property("display", wl_window.display())
                .build(),
            None => gtk::Window::new(),
        }
    }
}

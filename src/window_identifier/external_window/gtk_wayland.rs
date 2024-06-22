use gdk4wayland as gdk_wayland;
use gtk4::{gdk, prelude::*};

pub struct ExternalWaylandWindow {
    exported_handle: String,
    pub wl_display: gdk_wayland::WaylandDisplay,
}

impl ExternalWaylandWindow {
    pub fn new(exported_handle: String) -> Option<Self> {
        let display = {
            gdk::set_allowed_backends("wayland");
            let display = gdk::Display::open(None);
            gdk::set_allowed_backends("*");
            display.and_downcast::<gdk_wayland::WaylandDisplay>()
        };
        match display {
            Some(wl_display) => Some(Self {
                exported_handle,
                wl_display,
            }),
            None => {
                #[cfg(feature = "tracing")]
                tracing::warn!("Failed to open Wayland display");
                None
            }
        }
    }

    pub fn set_parent_of(&self, surface: &gdk_wayland::WaylandSurface) {
        if !surface
            .downcast_ref::<gdk_wayland::WaylandToplevel>()
            .unwrap()
            .set_transient_for_exported(&self.exported_handle)
        {
            #[cfg(feature = "tracing")]
            tracing::warn!("Failed to set portal window transient for external parent");
        }
    }

    pub fn display(&self) -> &gdk::Display {
        self.wl_display.upcast_ref()
    }
}

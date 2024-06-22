use std::os::raw::{c_uchar, c_ulong};

use gdk4x11 as gdk_x11;
use gdk_x11::x11::xlib;
use gtk4::{gdk, prelude::*};

pub struct ExternalX11Window {
    foreign_xid: c_ulong,
    pub x11_display: gdk_x11::X11Display,
}

impl ExternalX11Window {
    pub fn new(foreign_xid: c_ulong) -> Option<Self> {
        let display = {
            gdk::set_allowed_backends("x11");
            let display = gdk::Display::open(None);
            gdk::set_allowed_backends("*");
            display.and_downcast::<gdk_x11::X11Display>()
        };
        match display {
            Some(x11_display) => Some(Self {
                foreign_xid,
                x11_display,
            }),
            None => {
                #[cfg(feature = "tracing")]
                tracing::warn!("Failed to open X11 display");
                None
            }
        }
    }

    pub fn set_parent_of(&self, surface: &gdk_x11::X11Surface) {
        unsafe {
            let display = &self.x11_display;
            let x_display = display.xdisplay();
            let foreign_xid = self.foreign_xid;
            xlib::XSetTransientForHint(x_display, surface.xid(), foreign_xid);
            let atom =
                gdk_x11::x11_get_xatom_by_name_for_display(display, "_NET_WM_WINDOW_TYPE_DIALOG");
            xlib::XChangeProperty(
                x_display,
                surface.xid(),
                gdk_x11::x11_get_xatom_by_name_for_display(display, "_NET_WM_WINDOW_TYPE"),
                xlib::XA_ATOM,
                32,
                xlib::PropModeReplace,
                &atom as *const _ as *const c_uchar,
                1,
            );
        }
    }

    pub fn display(&self) -> &gdk::Display {
        self.x11_display.upcast_ref()
    }
}

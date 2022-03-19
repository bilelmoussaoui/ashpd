use super::{wayland, x11, WindowType};

use futures::lock::Mutex;
use gdk::Backend;
#[cfg(feature = "raw_handle")]
use glib::translate::ToGlibPtr;
use gtk3::{
    gdk,
    glib::{self, clone},
    prelude::*,
};
#[cfg(feature = "raw_handle")]
use raw_window_handle::{RawWindowHandle, WaylandHandle, XlibHandle};
use std::fmt;
use std::sync::Arc;

static WINDOW_HANDLE_KEY: &str = "ashpd-wayland-gtk3-window-handle";

pub struct Gtk3WindowIdentifier {
    window: gdk::Window,
    type_: WindowType,
}

impl Gtk3WindowIdentifier {
    pub async fn new(window: &impl glib::IsA<gdk::Window>) -> Option<Self> {
        match window.as_ref().display().backend() {
            Backend::Wayland => {
                let wayland_win = window
                    .as_ref()
                    .downcast_ref::<gdk3wayland::WaylandWindow>()
                    .unwrap();
                let handle = unsafe {
                    if let Some(mut handle) = wayland_win.data(WINDOW_HANDLE_KEY) {
                        let (handle, ref_count): &mut (Option<String>, u8) = handle.as_mut();
                        *ref_count += 1;
                        handle.clone()
                    } else {
                        let (sender, receiver) = futures::channel::oneshot::channel::<String>();
                        let sender = Arc::new(Mutex::new(Some(sender)));

                        wayland_win.export_handle(clone!(@strong sender => move |_, handle| {
                            let ctx = glib::MainContext::default();
                            let handle = handle.to_owned();
                            ctx.spawn_local(clone!(@strong sender, @strong handle => async move {
                                if let Some(m) = sender.lock().await.take() {
                                    let _ = m.send(handle);
                                }
                            }));
                        }));

                        let handle = receiver.await.ok();
                        wayland_win.set_data(WINDOW_HANDLE_KEY, (handle.clone(), 1));
                        handle
                    }
                };
                Some(Self {
                    window: window.clone().upcast(),
                    type_: WindowType::Wayland(handle.unwrap_or_default()),
                })
            }
            Backend::X11 => {
                let xid = window
                    .as_ref()
                    .downcast_ref::<gdk3x11::X11Window>()
                    .map(|w| w.xid())?;
                Some(Self {
                    window: window.clone().upcast(),
                    type_: WindowType::X11(xid),
                })
            }
            _ => None,
        }
    }

    #[cfg(feature = "raw_handle")]
    pub fn as_raw_handle(&self) -> RawWindowHandle {
        let display = self.window.display();
        unsafe {
            match self.type_ {
                WindowType::Wayland(_) => {
                    let mut wayland_handle = WaylandHandle::empty();
                    wayland_handle.surface = gdk3wayland::ffi::gdk_wayland_window_get_wl_surface(
                        self.window
                            .downcast_ref::<gdk3wayland::WaylandWindow>()
                            .unwrap()
                            .to_glib_none()
                            .0,
                    );
                    wayland_handle.display = gdk3wayland::ffi::gdk_wayland_display_get_wl_display(
                        display
                            .downcast_ref::<gdk3wayland::WaylandDisplay>()
                            .unwrap()
                            .to_glib_none()
                            .0,
                    );
                    RawWindowHandle::Wayland(wayland_handle)
                }
                WindowType::X11(xid) => {
                    let mut x11_handle = XlibHandle::empty();
                    x11_handle.window = xid;
                    x11_handle.display = gdk3x11::ffi::gdk_x11_display_get_xdisplay(
                        display
                            .downcast_ref::<gdk3x11::X11Display>()
                            .unwrap()
                            .to_glib_none()
                            .0,
                    ) as *mut _;
                    RawWindowHandle::Xlib(x11_handle)
                }
            }
        }
    }
}

impl fmt::Display for Gtk3WindowIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.type_ {
            WindowType::Wayland(handle) => f.write_str(&wayland::to_handle(&handle)),
            WindowType::X11(xid) => f.write_str(&x11::to_handle(*xid)),
        }
    }
}

impl Drop for Gtk3WindowIdentifier {
    fn drop(&mut self) {
        match self.type_ {
            WindowType::Wayland(_) => unsafe {
                let wayland_win = self
                    .window
                    .downcast_ref::<gdk3wayland::WaylandWindow>()
                    .unwrap();

                let (_handle, ref_count): &mut (Option<String>, u8) =
                    wayland_win.data(WINDOW_HANDLE_KEY).unwrap().as_mut();
                if ref_count > &mut 1 {
                    *ref_count -= 1;
                    return;
                }
                wayland_win.unexport_handle();
                let _ = wayland_win.steal_data::<(Option<String>, u8)>(WINDOW_HANDLE_KEY);
            },
            _ => (),
        }
    }
}

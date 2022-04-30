use std::{fmt, sync::Arc};

use futures::lock::Mutex;
use gdk::Backend;
#[cfg(feature = "raw_handle")]
use glib::translate::ToGlibPtr;
use gtk4::{
    gdk,
    glib::{self, clone},
    prelude::*,
};
#[cfg(feature = "raw_handle")]
use raw_window_handle::{RawWindowHandle, WaylandHandle, XlibHandle};

use super::WindowIdentifierType;

static WINDOW_HANDLE_KEY: &str = "ashpd-wayland-gtk4-window-handle";

pub struct Gtk4WindowIdentifier {
    native: gtk4::Native,
    type_: WindowIdentifierType,
}

impl Gtk4WindowIdentifier {
    pub async fn new(native: &impl glib::IsA<gtk4::Native>) -> Option<Self> {
        let surface = native.surface();
        match surface.display().backend() {
            Backend::Wayland => {
                let top_level = surface
                    .downcast_ref::<gdk4wayland::WaylandToplevel>()
                    .unwrap();
                let handle = unsafe {
                    if let Some(mut handle) = top_level.data(WINDOW_HANDLE_KEY) {
                        let (handle, ref_count): &mut (Option<String>, u8) = handle.as_mut();
                        *ref_count += 1;
                        handle.clone()
                    } else {
                        let (sender, receiver) = futures::channel::oneshot::channel::<String>();
                        let sender = Arc::new(Mutex::new(Some(sender)));

                        top_level.export_handle(clone!(@strong sender => move |_, handle| {
                            let ctx = glib::MainContext::default();
                            let handle = handle.to_owned();
                            ctx.spawn_local(clone!(@strong sender, @strong handle => async move {
                                if let Some(m) = sender.lock().await.take() {
                                    let _ = m.send(handle);
                                }
                            }));
                        }));
                        let handle = receiver.await.ok();
                        top_level.set_data(WINDOW_HANDLE_KEY, (handle.clone(), 1));
                        handle
                    }
                };
                Some(Gtk4WindowIdentifier {
                    native: native.clone().upcast(),
                    type_: WindowIdentifierType::Wayland(handle.unwrap_or_default()),
                })
            }
            Backend::X11 => {
                let xid = surface
                    .downcast_ref::<gdk4x11::X11Surface>()
                    .map(|w| w.xid())?;
                Some(Gtk4WindowIdentifier {
                    native: native.clone().upcast(),
                    type_: WindowIdentifierType::X11(xid),
                })
            }
            _ => None,
        }
    }

    #[cfg(feature = "raw_handle")]
    pub fn as_raw_handle(&self) -> RawWindowHandle {
        let surface = self.native.surface();
        let display = surface.display();
        unsafe {
            match self.type_ {
                WindowIdentifierType::Wayland(_) => {
                    let mut wayland_handle = WaylandHandle::empty();
                    wayland_handle.surface = gdk4wayland::ffi::gdk_wayland_surface_get_wl_surface(
                        surface
                            .downcast_ref::<gdk4wayland::WaylandSurface>()
                            .unwrap()
                            .to_glib_none()
                            .0,
                    );
                    wayland_handle.display = gdk4wayland::ffi::gdk_wayland_display_get_wl_display(
                        display
                            .downcast_ref::<gdk4wayland::WaylandDisplay>()
                            .unwrap()
                            .to_glib_none()
                            .0,
                    );
                    RawWindowHandle::Wayland(wayland_handle)
                }
                WindowIdentifierType::X11(xid) => {
                    let mut x11_handle = XlibHandle::empty();
                    x11_handle.window = xid;
                    x11_handle.display = gdk4x11::ffi::gdk_x11_display_get_xdisplay(
                        display
                            .downcast_ref::<gdk4x11::X11Display>()
                            .unwrap()
                            .to_glib_none()
                            .0,
                    );
                    RawWindowHandle::Xlib(x11_handle)
                }
            }
        }
    }
}

impl fmt::Display for Gtk4WindowIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}", self.type_))
    }
}

impl Drop for Gtk4WindowIdentifier {
    fn drop(&mut self) {
        match self.type_ {
            WindowIdentifierType::Wayland(_) => {
                let surface = self.native.surface();
                let top_level = surface
                    .downcast_ref::<gdk4wayland::WaylandToplevel>()
                    .unwrap();
                unsafe {
                    let (_handle, ref_count): &mut (Option<String>, u8) =
                        top_level.data(WINDOW_HANDLE_KEY).unwrap().as_mut();
                    if ref_count > &mut 1 {
                        *ref_count -= 1;
                        return;
                    }
                    top_level.unexport_handle();
                    let _ = top_level.steal_data::<(Option<String>, u8)>(WINDOW_HANDLE_KEY);
                }
            }
            _ => (),
        }
    }
}

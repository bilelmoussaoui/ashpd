#[cfg(feature = "wayland")]
use super::wayland::WaylandActivationToken;
use gdk4wayland::prelude::WaylandSurfaceExtManual;
use gtk4::{gdk, glib, prelude::*};

#[derive(Debug)]
pub struct Gtk4ActivationToken {
    pub(crate) wl_token: WaylandActivationToken,
}

#[cfg(all(feature = "gtk4_wayland", feature = "wayland"))]
impl Gtk4ActivationToken {
    pub async fn from_native<N: glib::IsA<gtk4::Native>>(app_id: &str, native: &N) -> Option<Self> {
        let surface = native.surface();
        match surface.display().backend() {
            gdk::Backend::Wayland => {
                let surface = surface
                    .downcast_ref::<gdk4wayland::WaylandSurface>()
                    .unwrap();
                if let Some(wl_surface) = surface.wl_surface() {
                    let wl_token = WaylandActivationToken::from_surface(app_id, &wl_surface)
                        .await
                        .unwrap();

                    Some(Self { wl_token })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(feature = "wayland")]
impl From<WaylandActivationToken> for Gtk4ActivationToken {
    fn from(wl_token: WaylandActivationToken) -> Self {
        Self { wl_token }
    }
}

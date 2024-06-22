mod gtk;
#[cfg(all(feature = "backend", feature = "gtk4_wayland"))]
mod gtk_wayland;

#[cfg(all(feature = "backend", feature = "gtk4_x11"))]
mod gtk_x11;

#[cfg(all(
    feature = "backend",
    any(feature = "gtk4_wayland", feature = "gtk4_x11")
))]
pub use self::gtk::GtkExternalWindow as ExternalWindow;

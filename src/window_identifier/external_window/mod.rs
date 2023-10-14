mod gtk;
mod gtk_wayland;
mod gtk_x11;

#[cfg(any(feature = "backend_gtk3", feature = "backend_gtk4"))]
pub use self::gtk::GtkExternalWindow as ExternalWindow;

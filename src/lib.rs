///! # libportal
///!
///! libportal is a Rust wrapper around the XDG Portals DBus interfaces
///! Specifications: [https://flatpak.github.io/xdg-desktop-portal/portal-docs.html](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html)
///! C alternative: [https://github.com/flatpak/libportal](https://github.com/flatpak/libportal)
///!
mod request;
mod session;

pub mod desktop;
pub mod documents;
pub mod flatpak;
pub use self::request::{RequestProxy, ResponseType};
pub use self::session::SessionProxy;
pub use zbus;
pub use zvariant;

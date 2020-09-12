//! libportal, the portal gun getting oxedized, the Rust wrapper for the XDG portals DBus interfaces.
//! ```no_run
//! use libportal::desktop::screenshot::{ScreenshotProxy, PickColorOptions};
//! use libportal::WindowIdentifier;
//!
//! fn main() -> zbus::fdo::Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = ScreenshotProxy::new(&connection)?;
//!     let request = proxy.pick_color(WindowIdentifier::default(), PickColorOptions::default())?;
//!     Ok(())
//! }
//! ```

/// Implementation of the various portals under `/org/freedesktop/portal/desktop`
pub mod desktop;
/// Implementation of the various portals under `/org/freedesktop/portal/documents`
pub mod documents;
/// Implementation of the various portals under `/org/freedesktop/portal/Flatpak`
pub mod flatpak;
///! # libportal
///!
///! libportal is a Rust wrapper around the XDG Portals DBus interfaces
///! Specifications: [https://flatpak.github.io/xdg-desktop-portal/portal-docs.html](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html)
///! C alternative: [https://github.com/flatpak/libportal](https://github.com/flatpak/libportal)
///!
mod request;
mod session;
mod window_identifier;
pub use self::request::{BasicResponse, RequestProxy, Response};
pub use self::session::SessionProxy;
pub use self::window_identifier::WindowIdentifier;
pub use zbus;
pub use zvariant;

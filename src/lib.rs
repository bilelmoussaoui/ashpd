#![deny(broken_intra_doc_links)]
#![deny(missing_docs)]
//! ASHPD, acronym of Aperture Science Handheld Portal Device is a Rust & [zbus](https://gitlab.freedesktop.org/zeenix/zbus) wrapper of
//! the XDG portals DBus interfaces. The library aims to provide an easy way to
//! interact with the various portals defined per the [specifications](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html).
//!
//! It provides an alternative to the C library <https://github.com/flatpak/libportal>.
//!
//! ```rust,no_run
//! use ashpd::{desktop::screenshot, Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! async fn run() -> Result<()> {
//!     let identifier = WindowIdentifier::default();
//!     if let Ok(Response::Ok(color)) = screenshot::pick_color(identifier).await {
//!         println!("({}, {}, {})", color.red(), color.green(), color.blue());
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ```rust,no_run
//! use ashpd::{desktop::camera, Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! async fn run() -> Result<()> {
//!     let identifier = WindowIdentifier::default();
//!     if let Ok(Response::Ok(pipewire_node_id)) = camera::stream().await {
//!         // Render the stream with GStreamer for example, see the demo
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Optional features
//!
//! | Feature | Description |
//! | ---     | ----------- |
//! | feature_gtk3 | Implement `Into<gdk3::RGBA>` for [`Color`] |
//! |  | Implement `From<gtk3::Window>` for [`WindowIdentifier`] |
//! | feature_gtk4 | Implement `Into<gdk4::RGBA>` for [`Color`] |
//! |  | Provides ['WindowIdentifier::from_window] |
//!
//!
//! [`Color`]: ./desktop/screenshot/struct.Color.html
//! [`WindowIdentifier`]: ./window_identifier/struct.WindowIdentifier.html
#[cfg(all(all(feature = "feature_gtk3", feature = "feature_gtk4"), not(doc)))]
compile_error!("You can't enable both GTK 3 & GTK 4 features at once");

/// Interact with the user's desktop such as taking a screenshot, setting a
/// background or querying the user's location.
pub mod desktop;
/// Interact with the documents store or transfer files across apps.
pub mod documents;
/// Spawn commands outside the sandbox or monitor if the running application has
/// received an update & install it.
pub mod flatpak;
mod handle_token;
mod request;
mod session;
mod window_identifier;
pub use enumflags2;
pub use zbus;
pub use zvariant;

/// Check whether the application is running inside a sandbox.
///
/// **Note** The check is very stupid as is for now.
pub fn is_sandboxed() -> bool {
    std::path::Path::new("/.flatpak-info").exists()
}

pub use self::handle_token::HandleToken;
pub use self::request::{AsyncRequestProxy, BasicResponse, RequestProxy, Response, ResponseError};
pub use self::session::{AsyncSessionProxy, SessionProxy};
pub use self::window_identifier::WindowIdentifier;

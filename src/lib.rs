#![deny(broken_intra_doc_links)]
//#![deny(missing_docs)]
//! ASHPD, acronym of Aperture Science Handheld Portal Device is a Rust & [zbus](https://gitlab.freedesktop.org/zeenix/zbus) wrapper of
//! the XDG portals DBus interfaces. The library aims to provide an easy way to
//! interact with the various portals defined per the [specifications](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html).
//!
//! It provides an alternative to the C library <https://github.com/flatpak/libportal>.
//!
//! # Examples
//!
//! Ask the compositor to pick a color
//! ```rust,no_run
//! use ashpd::{desktop::screenshot, WindowIdentifier};
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let identifier = WindowIdentifier::default();
//!     let color = screenshot::pick_color(identifier).await?;
//!     println!("({}, {}, {})", color.red(), color.green(), color.blue());
//!     Ok(())
//! }
//! ```
//!
//! Start a PipeWire stream from the user's camera
//! ```rust,no_run
//! use ashpd::{desktop::camera, WindowIdentifier};
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let identifier = WindowIdentifier::default();
//!     let pipewire_fd = camera::stream().await?;
//!     // Render the stream with GStreamer for example, see the demo
//!
//!     Ok(())
//! }
//! ```
//!
//! For a tour of the various portals, see the ASHPD demo application.
//!
//! # Optional features
//!
//! | Feature | Description |
//! | ---     | ----------- |
//! | feature_gtk3 | Implement `From<Color>` for `gdk3::RGBA` |
//! |  | Implement `From<gtk3::Window>` for [`WindowIdentifier`] |
//! | feature_gtk4 | Implement `From<Color>` for `gdk4::RGBA` |
//! |  | Provides `WindowIdentifier::from_window` |
#[cfg(all(all(feature = "feature_gtk3", feature = "feature_gtk4"), not(doc)))]
compile_error!("You can't enable both GTK 3 & GTK 4 features at once");

/// Interact with the user's desktop such as taking a screenshot, setting a
/// background or querying the user's location.
pub mod desktop;
/// Interact with the documents store or transfer files across apps.
pub mod documents;
mod error;
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

pub use self::error::Error;
pub use self::handle_token::HandleToken;
pub use self::request::{BasicResponse, RequestProxy, ResponseError};
pub use self::session::SessionProxy;
pub use self::window_identifier::WindowIdentifier;

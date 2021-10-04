#![deny(rustdoc::broken_intra_doc_links)]
#![allow(missing_docs)] // until zbus::DBusProxy adds docs to the generated trait
//! ASHPD, acronym of Aperture Science Handheld Portal Device is a Rust & [zbus](https://gitlab.freedesktop.org/dbus/zbus) wrapper of
//! the XDG portals DBus interfaces. The library aims to provide an easy way to
//! interact with the various portals defined per the [specifications](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html).
//!
//! It provides an alternative to the C library <https://github.com/flatpak/libportal>.
//!
//! # Examples
//!
//! Ask the compositor to pick a color
//! ```rust,no_run
//! use ashpd::desktop::screenshot::ScreenshotProxy;
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!
//!     let proxy = ScreenshotProxy::new(&connection).await?;
//!     let color = proxy.pick_color(&WindowIdentifier::default()).await?;
//!
//!     println!("({}, {}, {})", color.red(), color.green(), color.blue());
//!
//!     Ok(())
//! }
//! ```
//!
//! Start a PipeWire stream from the user's camera
//! ```rust,no_run
//! use ashpd::desktop::camera::CameraProxy;
//!
//! pub async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = CameraProxy::new(&connection).await?;
//!     if proxy.is_camera_present().await? {
//!         proxy.access_camera().await?;
//!
//!         let remote_fd = proxy.open_pipe_wire_remote().await?;
//!         // pass the remote fd to GStreamer for example
//!     }
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
//! | feature_gtk3 | Implement From<[Color](desktop::screenshot::Color)> for [`gdk3::RGBA`](https://gtk-rs.org/gtk3-rs/stable/latest/docs/gdk/struct.RGBA.html) |
//! |  | Provides `WindowIdentifier::from_window` that takes a [`IsA<gdk3::Window>`](https://gtk-rs.org/gtk3-rs/stable/latest/docs/gdk/struct.Window.html) |
//! | feature_gtk4 | Implement From<[Color](desktop::screenshot::Color)> for [`gdk4::RGBA`](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gdk4/struct.RGBA.html) |
//! |  | Provides `WindowIdentifier::from_native` that takes a [`IsA<gtk4::Native>`](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Native.html) |
//! | feature_pipewire  | Provides `ashpd::desktop::camera::pipewire_node_id` that helps you retrieve the PipeWire Node ID to use with the file descriptor returned by the camera portal |
#[cfg(all(all(feature = "feature_gtk3", feature = "feature_gtk4"), not(doc)))]
compile_error!("You can't enable both GTK 3 & GTK 4 features at once");

/// Alias for a [`Result`] with the error type `ashpd::Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// Interact with the user's desktop such as taking a screenshot, setting a
/// background or querying the user's location.
pub mod desktop;
/// Interact with the documents store or transfer files across apps.
pub mod documents;
mod error;
mod window_identifier;
pub use self::window_identifier::WindowIdentifier;
/// Spawn commands outside the sandbox or monitor if the running application has
/// received an update & install it.
pub mod flatpak;
mod helpers;
pub use enumflags2;
pub use zbus;
pub use zvariant;

/// Check whether the application is running inside a sandbox.
///
/// **Note** The check is very stupid as is for now.
pub fn is_sandboxed() -> bool {
    std::path::Path::new("/.flatpak-info").exists()
}

pub use self::error::{Error, PortalError};

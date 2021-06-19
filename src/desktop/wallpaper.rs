//! # Examples
//!
//! ## Sets a wallpaper from a file:
//!
//! ```rust,no_run
//! use ashpd::{
//!     desktop::wallpaper::{SetOn, WallpaperOptions, WallpaperProxy},
//!     WindowIdentifier,
//! };
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let identifier = WindowIdentifier::default();
//!     let wallpaper =
//!         File::open("/home/bilelmoussaoui/adwaita-day.jpg").expect("wallpaper not found");
//!
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = WallpaperProxy::new(&connection).await?;
//!     proxy
//!         .set_wallpaper_file(
//!             identifier,
//!             wallpaper.as_raw_fd(),
//!             WallpaperOptions::default()
//!                 .show_preview(true)
//!                 .set_on(SetOn::Both),
//!         )
//!         .await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Sets a wallpaper from a URI:
//!
//! ```rust,no_run
//! use ashpd::{
//!     desktop::wallpaper::{SetOn, WallpaperOptions, WallpaperProxy},
//!     WindowIdentifier,
//! };
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let identifier = WindowIdentifier::default();
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = WallpaperProxy::new(&connection).await?;
//!     proxy
//!         .set_wallpaper_uri(
//!             identifier,
//!             "file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg",
//!             WallpaperOptions::default()
//!                 .show_preview(true)
//!                 .set_on(SetOn::Both),
//!         )
//!         .await?;
//!     Ok(())
//! }
//! ```
use serde::{self, Deserialize, Serialize, Serializer};
use std::os::unix::prelude::AsRawFd;
use strum_macros::{AsRefStr, EnumString, IntoStaticStr, ToString};
use zvariant::{Signature, Type};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{
    helpers::{call_basic_response_method, property},
    Error, WindowIdentifier,
};

#[derive(
    Deserialize, Debug, Clone, Copy, PartialEq, Hash, AsRefStr, EnumString, IntoStaticStr, ToString,
)]
#[serde(rename = "lowercase")]
/// Where to set the wallpaper on.
pub enum SetOn {
    /// Set the wallpaper only on the lock-screen.
    Lockscreen,
    /// Set the wallpaper only on the background.
    Background,
    /// Set the wallpaper on both lock-screen and background.
    Both,
}

impl Type for SetOn {
    fn signature() -> Signature<'static> {
        String::signature()
    }
}

impl Serialize for SetOn {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        String::serialize(&self.to_string(), serializer)
    }
}

#[derive(SerializeDict, DeserializeDict, Clone, TypeDict, Debug, Default)]
/// Specified options for a [`WallpaperProxy::set_wallpaper_file`] or a [`WallpaperProxy::set_wallpaper_uri`] request.
pub struct WallpaperOptions {
    /// Whether to show a preview of the picture
    #[zvariant(rename = "show-preview")]
    show_preview: Option<bool>,
    /// Where to set the wallpaper on
    #[zvariant(rename = "set-on")]
    set_on: Option<SetOn>,
}

impl WallpaperOptions {
    /// Whether to show a preview of the picture.
    /// **Note** that the portal may decide to show a preview even if this
    /// option is not set.
    pub fn show_preview(mut self, show_preview: bool) -> Self {
        self.show_preview = Some(show_preview);
        self
    }

    /// Sets where to set the wallpaper on.
    pub fn set_on(mut self, set_on: SetOn) -> Self {
        self.set_on = Some(set_on);
        self
    }
}

/// The interface lets sandboxed applications set the user's desktop background
/// picture.
#[derive(Debug)]
pub struct WallpaperProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> WallpaperProxy<'a> {
    /// Create a new instance of [`WallpaperProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<WallpaperProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Wallpaper")
            .path("/org/freedesktop/portal/desktop")?
            .destination("org.freedesktop.portal.Desktop")
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Sets the lock-screen, background or both wallpaper's from a file
    /// descriptor.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `fd` - The wallpaper file description.
    /// * `options` - A [`WallpaperOptions`].
    pub async fn set_wallpaper_file<F>(
        &self,
        parent_window: WindowIdentifier,
        fd: F,
        options: WallpaperOptions,
    ) -> Result<(), Error>
    where
        F: AsRawFd + Type + Serialize,
    {
        call_basic_response_method(
            &self.0,
            "SetWallpaperFile",
            &(parent_window, fd.as_raw_fd(), options),
        )
        .await
    }

    /// Sets the lock-screen, background or both wallpaper's from an URI.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `uri` - The wallpaper URI.
    /// * `options` - A [`WallpaperOptions`].
    pub async fn set_wallpaper_uri(
        &self,
        parent_window: WindowIdentifier,
        uri: &str,
        options: WallpaperOptions,
    ) -> Result<(), Error> {
        call_basic_response_method(&self.0, "SetWallpaperURI", &(parent_window, uri, options)).await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}

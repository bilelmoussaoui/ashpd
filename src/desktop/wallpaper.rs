//! # Examples
//!
//! Sets a wallpaper from a file:
//!
//! ```rust,no_run
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//! use ashpd::{desktop::wallpaper, WindowIdentifier};
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let identifier = WindowIdentifier::default();
//!     let wallpaper =
//!         File::open("/home/bilelmoussaoui/adwaita-day.jpg").expect("wallpaper not found");
//!
//!     if wallpaper::set_from_file(
//!         identifier,
//!         wallpaper.as_raw_fd(),
//!         true,
//!         wallpaper::SetOn::Both,
//!     ).await.is_ok()
//!     {
//!         // wallpaper was set successfully
//!     }
//!     Ok(())
//! }
//! ```
//!
//! Sets a wallpaper from a URI:
//!
//! ```rust,no_run
//! use ashpd::{desktop::wallpaper, WindowIdentifier};
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let identifier = WindowIdentifier::default();
//!     if wallpaper::set_from_uri(
//!         identifier,
//!         "file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg",
//!         true,
//!         wallpaper::SetOn::Both,
//!     ).await.is_ok()
//!     {
//!         // wallpaper was set successfully
//!     }
//!     Ok(())
//! }
//! ```
use serde::{self, Deserialize, Serialize, Serializer};
use std::os::unix::prelude::AsRawFd;
use strum_macros::{AsRefStr, EnumString, IntoStaticStr, ToString};
use zvariant::{Signature, Type};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{BasicResponse, Error, RequestProxy, WindowIdentifier};

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
/// Specified options for a set wallpaper request.
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
pub struct WallpaperProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> WallpaperProxy<'a> {
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
    ///
    /// [`WallpaperOptions`]: ./struct.WallpaperOptions.html
    pub async fn set_wallpaper_file<F>(
        &self,
        parent_window: WindowIdentifier,
        fd: F,
        options: WallpaperOptions,
    ) -> Result<RequestProxy<'a>, Error>
    where
        F: AsRawFd + Type + Serialize,
    {
        let path: zvariant::OwnedObjectPath = self
            .0
            .call_method("AccessDevice", &(parent_window, fd.as_raw_fd(), options))
            .await?
            .body()?;
        RequestProxy::new(self.0.connection(), path).await
    }

    /// Sets the lock-screen, background or both wallpaper's from an URI.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `uri` - The wallpaper URI.
    /// * `options` - A [`WallpaperOptions`].
    ///
    /// [`WallpaperOptions`]: ./struct.WallpaperOptions.html
    pub async fn set_wallpaper_uri(
        &self,
        parent_window: WindowIdentifier,
        uri: &str,
        options: WallpaperOptions,
    ) -> Result<RequestProxy<'a>, Error> {
        let path: zvariant::OwnedObjectPath = self
            .0
            .call_method("c", &(parent_window, uri, options))
            .await?
            .body()?;
        RequestProxy::new(self.0.connection(), path).await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        self.0
            .get_property::<u32>("version")
            .await
            .map_err(From::from)
    }
}

/// Set a wallpaper from a file.
///
/// An async function around the `WallpaperProxy::set_wallpaper_file`.
pub async fn set_from_file<F: AsRawFd + Type + Serialize>(
    window_identifier: WindowIdentifier,
    wallpaper_file: F,
    show_preview: bool,
    set_on: SetOn,
) -> Result<(), Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = WallpaperProxy::new(&connection).await?;
    let request = proxy
        .set_wallpaper_file(
            window_identifier,
            wallpaper_file,
            WallpaperOptions::default()
                .show_preview(show_preview)
                .set_on(set_on),
        )
        .await?;
    let _wallpaper = request.receive_response::<BasicResponse>().await?;
    Ok(())
}

/// Set a wallpaper from a URI.
///
/// An async function around the `WallpaperProxy::set_wallpaper_uri`.
pub async fn set_from_uri(
    window_identifier: WindowIdentifier,
    wallpaper_uri: &str,
    show_preview: bool,
    set_on: SetOn,
) -> Result<(), Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = WallpaperProxy::new(&connection).await?;
    let request = proxy
        .set_wallpaper_uri(
            window_identifier,
            wallpaper_uri,
            WallpaperOptions::default()
                .show_preview(show_preview)
                .set_on(set_on),
        )
        .await?;
    let _wallpaper = request.receive_response::<BasicResponse>().await?;
    Ok(())
}

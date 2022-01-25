//! # Examples
//!
//! ## Sets a wallpaper from a file:
//!
//!```rust,no_run
//! use ashpd::desktop::wallpaper::{self, SetOn};
//! use ashpd::WindowIdentifier;
//! use std::fs::File;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let file = File::open("/home/bilelmoussaoui/adwaita-day.jpg").unwrap();
//!     wallpaper::set_from_file(&WindowIdentifier::default(), &file, true, SetOn::Both).await?;
//!     Ok(())
//! }
//! ```
//!
//! Or by using the Proxy directly
//!
//! ```rust,no_run
//! use ashpd::desktop::wallpaper::{SetOn, WallpaperProxy};
//! use ashpd::WindowIdentifier;
//! use std::fs::File;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let wallpaper = File::open("/home/bilelmoussaoui/adwaita-day.jpg").unwrap();
//!
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = WallpaperProxy::new(&connection).await?;
//!     proxy
//!         .set_wallpaper_file(&WindowIdentifier::default(), &wallpaper, true, SetOn::Both)
//!         .await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Sets a wallpaper from a URI:
//!
//!```rust,no_run
//! use ashpd::desktop::wallpaper::{self, SetOn};
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let uri = "file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg";
//!     wallpaper::set_from_uri(&WindowIdentifier::default(), &uri, true, SetOn::Both).await?;
//!     Ok(())
//! }
//! ```
//!
//! Or by using the Proxy directly
//!
//! ```rust,no_run
//! use ashpd::desktop::wallpaper::{SetOn, WallpaperProxy};
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = WallpaperProxy::new(&connection).await?;
//!     proxy
//!         .set_wallpaper_uri(
//!             &WindowIdentifier::default(),
//!             "file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg",
//!             true,
//!             SetOn::Both,
//!         )
//!         .await?;
//!     Ok(())
//! }
//! ```

use std::os::unix::prelude::AsRawFd;
use std::{fmt, str::FromStr};

use serde::{self, Deserialize, Serialize, Serializer};
use zbus::zvariant::{DeserializeDict, Fd, SerializeDict, Signature, Type};

use crate::{
    desktop::{HandleToken, DESTINATION, PATH},
    helpers::call_basic_response_method,
    Error, WindowIdentifier,
};

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Hash)]
/// Where to set the wallpaper on.
pub enum SetOn {
    /// Set the wallpaper only on the lock-screen.
    Lockscreen,
    /// Set the wallpaper only on the background.
    Background,
    /// Set the wallpaper on both lock-screen and background.
    Both,
}

impl fmt::Display for SetOn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lockscreen => write!(f, "Lockscreen"),
            Self::Background => write!(f, "Background"),
            Self::Both => write!(f, "Both"),
        }
    }
}

impl AsRef<str> for SetOn {
    fn as_ref(&self) -> &str {
        match self {
            Self::Lockscreen => "Lockscreen",
            Self::Background => "Background",
            Self::Both => "Both",
        }
    }
}

impl From<SetOn> for &'static str {
    fn from(s: SetOn) -> Self {
        match s {
            SetOn::Lockscreen => "Lockscreen",
            SetOn::Background => "Background",
            SetOn::Both => "Both",
        }
    }
}

impl FromStr for SetOn {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Lockscreen" => Ok(SetOn::Lockscreen),
            "Background" => Ok(SetOn::Background),
            "Both" => Ok(SetOn::Both),
            _ => Err(Error::ParseError(
                "Failed to parse SetOn, invalid value".to_string(),
            )),
        }
    }
}

impl Serialize for SetOn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string().to_lowercase())
    }
}

impl Type for SetOn {
    fn signature() -> Signature<'static> {
        String::signature()
    }
}

#[derive(SerializeDict, DeserializeDict, Clone, Type, Debug, Default)]
/// Specified options for a [`WallpaperProxy::set_wallpaper_file`] or a
/// [`WallpaperProxy::set_wallpaper_uri`] request.
#[zvariant(signature = "dict")]
struct WallpaperOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// Whether to show a preview of the picture
    #[zvariant(rename = "show-preview")]
    show_preview: Option<bool>,
    /// Where to set the wallpaper on
    #[zvariant(rename = "set-on")]
    set_on: Option<SetOn>,
}

impl WallpaperOptions {
    /// Whether to show a preview of the picture.
    /// **Note** the portal may decide to show a preview even if this option is
    /// not set.
    #[must_use]
    pub fn show_preview(mut self, show_preview: bool) -> Self {
        self.show_preview = Some(show_preview);
        self
    }

    /// Sets where to set the wallpaper on.
    #[must_use]
    pub fn set_on(mut self, set_on: SetOn) -> Self {
        self.set_on = Some(set_on);
        self
    }
}
/// The interface lets sandboxed applications set the user's desktop background
/// picture.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Wallpaper`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Wallpaper).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Wallpaper")]
pub struct WallpaperProxy<'a>(zbus::Proxy<'a>);

impl<'a> WallpaperProxy<'a> {
    /// Create a new instance of [`WallpaperProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<WallpaperProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Wallpaper")?
            .path(PATH)?
            .destination(DESTINATION)?
            .build()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::Proxy<'_> {
        &self.0
    }

    /// Sets the lock-screen, background or both wallpaper's from a file
    /// descriptor.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    /// * `file` - The wallpaper file descriptor.
    /// * `show_preview` - Whether to show a preview of the picture.
    /// * `set_on` - Where to set the wallpaper on.
    ///
    /// # Specifications
    ///
    /// See also [`SetWallpaperFile`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Wallpaper.SetWallpaperFile).
    #[doc(alias = "SetWallpaperFile")]
    #[doc(alias = "xdp_portal_set_wallpaper")]
    pub async fn set_wallpaper_file(
        &self,
        identifier: &WindowIdentifier,
        file: &impl AsRawFd,
        show_preview: bool,
        set_on: SetOn,
    ) -> Result<(), Error> {
        let options = WallpaperOptions::default()
            .show_preview(show_preview)
            .set_on(set_on);
        call_basic_response_method(
            self.inner(),
            &options.handle_token,
            "SetWallpaperFile",
            &(&identifier, Fd::from(file.as_raw_fd()), &options),
        )
        .await
    }

    /// Sets the lock-screen, background or both wallpaper's from an URI.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    /// * `uri` - The wallpaper URI.
    /// * `show_preview` - Whether to show a preview of the picture.
    /// * `set_on` - Where to set the wallpaper on.
    ///
    /// # Specifications
    ///
    /// See also [`SetWallpaperURI`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Wallpaper.SetWallpaperURI).
    #[doc(alias = "SetWallpaperURI")]
    #[doc(alias = "xdp_portal_set_wallpaper")]
    pub async fn set_wallpaper_uri(
        &self,
        identifier: &WindowIdentifier,
        uri: &str,
        show_preview: bool,
        set_on: SetOn,
    ) -> Result<(), Error> {
        let options = WallpaperOptions::default()
            .show_preview(show_preview)
            .set_on(set_on);
        call_basic_response_method(
            self.inner(),
            &options.handle_token,
            "SetWallpaperURI",
            &(&identifier, uri, &options),
        )
        .await
    }
}

#[doc(alias = "xdp_portal_set_wallpaper")]
/// A handy wrapper around [`WallpaperProxy::set_wallpaper_uri`].
pub async fn set_from_uri(
    identifier: &WindowIdentifier,
    uri: &str,
    show_preview: bool,
    set_on: SetOn,
) -> Result<(), Error> {
    let connection = zbus::Connection::session().await?;
    let proxy = WallpaperProxy::new(&connection).await?;
    proxy
        .set_wallpaper_uri(identifier, uri, show_preview, set_on)
        .await?;
    Ok(())
}

#[doc(alias = "xdp_portal_set_wallpaper")]
/// A handy wrapper around [`WallpaperProxy::set_wallpaper_file`].
pub async fn set_from_file(
    identifier: &WindowIdentifier,
    file: &impl AsRawFd,
    show_preview: bool,
    set_on: SetOn,
) -> Result<(), Error> {
    let connection = zbus::Connection::session().await?;
    let proxy = WallpaperProxy::new(&connection).await?;
    proxy
        .set_wallpaper_file(identifier, file, show_preview, set_on)
        .await?;
    Ok(())
}

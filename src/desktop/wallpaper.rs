//! # Examples
//!
//! ## Sets a wallpaper from a file:
//!
//! ```rust,no_run
//! use ashpd::desktop::wallpaper::{SetOn, WallpaperProxy};
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let wallpaper =
//!         File::open("/home/bilelmoussaoui/adwaita-day.jpg").expect("wallpaper not found");
//!
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = WallpaperProxy::new(&connection).await?;
//!     proxy
//!         .set_wallpaper_file(Default::default(), wallpaper.as_raw_fd(), true, SetOn::Both)
//!         .await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Sets a wallpaper from a URI:
//!
//! ```rust,no_run
//! use ashpd::desktop::wallpaper::{SetOn, WallpaperProxy};
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = WallpaperProxy::new(&connection).await?;
//!     proxy
//!         .set_wallpaper_uri(
//!             Default::default(),
//!             "file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg",
//!             true,
//!             SetOn::Both,
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
    desktop::{HandleToken, DESTINATION, PATH},
    helpers::call_basic_response_method,
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
#[doc(alias = "org.freedesktop.portal.Wallpaper")]
pub struct WallpaperProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> WallpaperProxy<'a> {
    /// Create a new instance of [`WallpaperProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<WallpaperProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Wallpaper")
            .path(PATH)?
            .destination(DESTINATION)
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::azync::Proxy<'_> {
        &self.0
    }

    /// Sets the lock-screen, background or both wallpaper's from a file
    /// descriptor.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    /// * `fd` - The wallpaper file description.
    /// * `show_preview` - Whether to show a preview of the picture.
    /// * `set_on` - Where to set the wallpaper on.
    #[doc(alias = "SetWallpaperFile")]
    pub async fn set_wallpaper_file<F>(
        &self,
        identifier: WindowIdentifier,
        fd: F,
        show_preview: bool,
        set_on: SetOn,
    ) -> Result<(), Error>
    where
        F: AsRawFd + Type + Serialize,
    {
        let options = WallpaperOptions::default()
            .show_preview(show_preview)
            .set_on(set_on);
        call_basic_response_method(
            &self.0,
            &options.handle_token,
            "SetWallpaperFile",
            &(identifier, fd.as_raw_fd(), &options),
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
    #[doc(alias = "SetWallpaperURI")]
    pub async fn set_wallpaper_uri(
        &self,
        identifier: WindowIdentifier,
        uri: &str,
        show_preview: bool,
        set_on: SetOn,
    ) -> Result<(), Error> {
        let options = WallpaperOptions::default()
            .show_preview(show_preview)
            .set_on(set_on);
        call_basic_response_method(
            &self.0,
            &options.handle_token,
            "SetWallpaperURI",
            &(identifier, uri, &options),
        )
        .await
    }
}

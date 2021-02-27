//! # Examples
//!
//! Sets a wallpaper from a file:
//!
//! ```rust,no_run
//! use ashpd::desktop::wallpaper::{SetOn, WallpaperOptions, WallpaperProxy};
//! use ashpd::{BasicResponse as Basic, RequestProxy, Response, WindowIdentifier};
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//! use zbus::fdo::Result;
//! use zvariant::Fd;
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = WallpaperProxy::new(&connection)?;
//!
//!     let wallpaper =
//!         File::open("/home/bilelmoussaoui/adwaita-day.jpg").expect("wallpaper not found");
//!
//!     let request = proxy.set_wallpaper_file(
//!         WindowIdentifier::default(),
//!         Fd::from(wallpaper.as_raw_fd()),
//!         WallpaperOptions::default().set_on(SetOn::Background),
//!     )?;
//!
//!     request.connect_response(|response: Response<Basic>| {
//!         println!("{}", response.is_ok());
//!         Ok(())
//!     })?;
//!     Ok(())
//! }
//! ```
//!
//! Sets a wallpaper from a URI:
//!
//! ```rust,no_run
//! use ashpd::desktop::wallpaper::{SetOn, WallpaperOptions, WallpaperProxy};
//! use ashpd::{BasicResponse as Basic, RequestProxy, Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = WallpaperProxy::new(&connection)?;
//!
//!     let request = proxy.set_wallpaper_uri(
//!         WindowIdentifier::default(),
//!         "file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg",
//!         WallpaperOptions::default()
//!             .show_preview(true)
//!             .set_on(SetOn::Both),
//!     )?;
//!
//!     request.connect_response(|response: Response<Basic>| {
//!         println!("{}", response.is_ok());
//!         Ok(())
//!     })?;
//!     Ok(())
//! }
//! ```
use serde::{self, Deserialize, Serialize, Serializer};
use strum_macros::{AsRefStr, EnumString, IntoStaticStr, ToString};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{Fd, Signature, Type};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{AsyncRequestProxy, RequestProxy, WindowIdentifier};

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

#[dbus_proxy(
    interface = "org.freedesktop.portal.Wallpaper",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications set the user's desktop background
/// picture.
trait Wallpaper {
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
    #[dbus_proxy(object = "Request")]
    fn set_wallpaper_file(
        &self,
        parent_window: WindowIdentifier,
        fd: Fd,
        options: WallpaperOptions,
    );

    /// Sets the lock-screen, background or both wallpaper's from an URI.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `uri` - The wallpaper URI.
    /// * `options` - A [`WallpaperOptions`].
    ///
    /// [`WallpaperOptions`]: ./struct.WallpaperOptions.html
    #[dbus_proxy(name = "SetWallpaperURI", object = "Request")]
    fn set_wallpaper_uri(
        &self,
        parent_window: WindowIdentifier,
        uri: &str,
        options: WallpaperOptions,
    );

    /// The version of this DBus interface.
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

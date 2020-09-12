//! # Examples
//!
//! Sets a wallpaper from a file:
//!
//! ```no_run
//! use libportal::desktop::wallpaper::{WallpaperOptions, WallpaperProxy, SetOn, WallpaperResponse};
//! use libportal::{RequestProxy, WindowIdentifier};
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//! use zbus::fdo::Result;
//! use zvariant::Fd;
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = WallpaperProxy::new(&connection)?;
//!
//!     let wallpaper = File::open("/home/bilelmoussaoui/adwaita-day.jpg").expect("wallpaper not found");
//!
//!     let request_handle = proxy.set_wallpaper_file(
//!         WindowIdentifier::default(),
//!         Fd::from(wallpaper.as_raw_fd()),
//!         WallpaperOptions::default()
//!             .set_on(SetOn::Background),
//!     )?;
//!
//!     let request = RequestProxy::new(&connection, &request_handle)?;
//!     request.on_response(|response: WallpaperResponse| -> Result<()> {
//!         println!("{}", response.is_success() );
//!         Ok(())
//!     })?;
//!     Ok(())
//! }
//! ```
//!
//! Sets a wallpaper from a URI:
//!
//! ```no_run
//! use libportal::desktop::wallpaper::{WallpaperOptions, WallpaperProxy, SetOn, WallpaperResponse};
//! use libportal::{RequestProxy, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! fn main() -> zbus::fdo::Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = WallpaperProxy::new(&connection)?;
//!
//!     let request_handle = proxy.set_wallpaper_uri(
//!         WindowIdentifier::default(),
//!         "file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg",
//!         WallpaperOptions::default()
//!             .show_preview(true)
//!             .set_on(SetOn::Both),
//!     )?;
//!
//!     let request = RequestProxy::new(&connection, &request_handle)?;
//!     request.on_response(|response: WallpaperResponse| -> Result<()> {
//!         println!("{}", response.is_success() );
//!         Ok(())
//!     })?;
//!     Ok(())
//! }
//! ```
use crate::{ResponseType, WindowIdentifier};
use serde::{self, Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use strum_macros::{AsRefStr, EnumString, IntoStaticStr, ToString};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{Fd, OwnedObjectPath, OwnedValue, Signature};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(Deserialize, Debug, Clone, Copy, AsRefStr, EnumString, IntoStaticStr, ToString)]
#[serde(rename = "lowercase")]
/// Where to set the wallpaper on.
pub enum SetOn {
    /// Set the wallpaper only on the lockscreen.
    Lockscreen,
    /// Set the wallpaper only on the background.
    Background,
    /// Set the wallpaper on both lockscreen and background.
    Both,
}

impl zvariant::Type for SetOn {
    fn signature() -> Signature<'static> {
        Signature::from_string_unchecked("s".to_string())
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

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a set wallpaper request.
pub struct WallpaperOptions {
    /// Whether to show a preview of the picture
    /// Note that the portal may decide to show a preview even if this option is not set
    #[zvariant(rename = "show-preview")]
    pub show_preview: Option<bool>,
    /// Where to set the wallpaper on
    #[zvariant(rename = "set-on")]
    pub set_on: Option<SetOn>,
}

impl WallpaperOptions {
    pub fn show_preview(mut self, show_preview: bool) -> Self {
        self.show_preview = Some(show_preview);
        self
    }

    pub fn set_on(mut self, set_on: SetOn) -> Self {
        self.set_on = Some(set_on);
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct WallpaperResponse(ResponseType, HashMap<String, OwnedValue>);

impl WallpaperResponse {
    pub fn is_success(&self) -> bool {
        self.0 == ResponseType::Success
    }
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Wallpaper",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications set the user's desktop background picture.
trait Wallpaper {
    /// Sets the lockscreen, background or both wallapers from a file descriptor
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `fd` - The wallapaper file description
    /// * `options` - A [`WallpaperOptions`]
    ///
    /// [`WallpaperOptions`]: ./struct.WallpaperOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn set_wallpaper_file(
        &self,
        parent_window: WindowIdentifier,
        fd: Fd,
        options: WallpaperOptions,
    ) -> Result<OwnedObjectPath>;

    /// Sets the lockscreen, background or both wallapers from an URI
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `uri` - The wallapaper URI
    /// * `options` - A [`WallpaperOptions`]
    ///
    /// [`WallpaperOptions`]: ./struct.WallpaperOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    #[dbus_proxy(name = "SetWallpaperURI")]
    fn set_wallpaper_uri(
        &self,
        parent_window: WindowIdentifier,
        uri: &str,
        options: WallpaperOptions,
    ) -> Result<OwnedObjectPath>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

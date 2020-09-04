use crate::WindowIdentifier;
use serde::{self, Deserialize, Serialize};
use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{OwnedObjectPath, Signature, Type};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename = "lowercase")]
/// Where to set the wallpaper on.
pub enum WallpaperSetOn {
    /// Set the wallpaper only on the lockscreen.
    Lockscreen,
    /// Set the wallpaper only on the background.
    Background,
    /// Set the wallpaper on both lockscreen and background.
    Both,
}

impl Type for WallpaperSetOn {
    fn signature() -> Signature<'static> {
        Signature::from_string_unchecked("s".to_string())
    }
}

impl std::convert::TryFrom<zvariant::Value<'_>> for WallpaperSetOn {
    type Error = zvariant::Error;
    fn try_from(v: zvariant::Value<'_>) -> std::result::Result<Self, Self::Error> {
        match v {
            zvariant::Value::Str(s) => match s.as_str() {
                "lockscreen" => Ok(WallpaperSetOn::Lockscreen),
                "background" => Ok(WallpaperSetOn::Background),
                "both" => Ok(WallpaperSetOn::Both),
                _ => Err(zvariant::Error::Message("invalid value".to_string())),
            },
            _ => Err(zvariant::Error::IncorrectType),
        }
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a set wallpaper request.
pub struct WallpaperOptions {
    /// Whether to show a preview of the picture
    /// Note that the portal may decide to show a preview even if this option is not set
    pub show_preview: Option<bool>,
    /// Where to set the wallpaper on
    pub set_on: Option<String>,
}

#[derive(Debug, Default)]
pub struct WallpaperOptionsBuilder {
    /// Whether to show a preview of the picture
    /// Note that the portal may decide to show a preview even if this option is not set
    pub show_preview: Option<bool>,
    /// Where to set the wallpaper on
    pub set_on: Option<String>,
}

impl WallpaperOptionsBuilder {
    pub fn show_preview(mut self, show_preview: bool) -> Self {
        self.show_preview = Some(show_preview);
        self
    }

    pub fn set_on(mut self, set_on: &str) -> Self {
        self.set_on = Some(set_on.to_string());
        self
    }

    pub fn build(self) -> WallpaperOptions {
        WallpaperOptions {
            set_on: self.set_on,
            show_preview: self.show_preview,
        }
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
        fd: RawFd,
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

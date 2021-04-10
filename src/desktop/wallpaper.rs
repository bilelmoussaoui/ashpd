//! # Examples
//!
//! Sets a wallpaper from a file:
//!
//! ```rust,no_run
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//! use ashpd::{desktop::wallpaper, Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! async fn run() -> Result<()> {
//!     let identifier = WindowIdentifier::default();
//!     let wallpaper =
//!         File::open("/home/bilelmoussaoui/adwaita-day.jpg").expect("wallpaper not found");
//!
//!     if let Response::Ok(_) = wallpaper::set_from_file(
//!         identifier,
//!         wallpaper.as_raw_fd(),
//!         true,
//!         wallpaper::SetOn::Both,
//!     )
//!     .await?
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
//! use ashpd::{desktop::wallpaper, Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! async fn run() -> Result<()> {
//!     let identifier = WindowIdentifier::default();
//!     if let Response::Ok(_) = wallpaper::set_from_uri(
//!         identifier,
//!         "file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg",
//!         true,
//!         wallpaper::SetOn::Both,
//!     )
//!     .await?
//!     {
//!         // wallpaper was set successfully
//!     }
//!     Ok(())
//! }
//! ```
use std::os::unix::prelude::AsRawFd;
use std::sync::Arc;

use futures::{lock::Mutex, FutureExt};
use serde::{self, Deserialize, Serialize, Serializer};
use strum_macros::{AsRefStr, EnumString, IntoStaticStr, ToString};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{Signature, Type};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{AsyncRequestProxy, BasicResponse, RequestProxy, Response, WindowIdentifier};

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
    fn set_wallpaper_file<F>(
        &self,
        parent_window: WindowIdentifier,
        fd: F,
        options: WallpaperOptions,
    ) where
        F: AsRawFd + Type + Serialize;

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

/// Set a wallpaper from a file.
///
/// An async function around the `AsyncWallpaperProxy::set_wallpaper_file`.
pub async fn set_from_file<F: AsRawFd + Type + Serialize>(
    window_identifier: WindowIdentifier,
    wallpaper_file: F,
    show_preview: bool,
    set_on: SetOn,
) -> zbus::Result<Response<BasicResponse>> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncWallpaperProxy::new(&connection)?;
    let request = proxy
        .set_wallpaper_file(
            window_identifier,
            wallpaper_file,
            WallpaperOptions::default()
                .show_preview(show_preview)
                .set_on(set_on),
        )
        .await?;

    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Arc::new(Mutex::new(Some(sender)));
    let signal_id = request
        .connect_response(move |response: Response<BasicResponse>| {
            let s = sender.clone();
            async move {
                if let Some(m) = s.lock().await.take() {
                    let _ = m.send(response);
                }
                Ok(())
            }
            .boxed()
        })
        .await?;

    while request.next_signal().await?.is_some() {}
    request.disconnect_signal(signal_id).await?;

    let wallpaper = receiver.await.unwrap();
    Ok(wallpaper)
}

/// Set a wallpaper from a URI.
///
/// An async function around the `AsyncWallpaperProxy::set_wallpaper_uri`.
pub async fn set_from_uri(
    window_identifier: WindowIdentifier,
    wallpaper_uri: &str,
    show_preview: bool,
    set_on: SetOn,
) -> zbus::Result<Response<BasicResponse>> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncWallpaperProxy::new(&connection)?;
    let request = proxy
        .set_wallpaper_uri(
            window_identifier,
            wallpaper_uri,
            WallpaperOptions::default()
                .show_preview(show_preview)
                .set_on(set_on),
        )
        .await?;

    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Arc::new(Mutex::new(Some(sender)));
    let signal_id = request
        .connect_response(move |response: Response<BasicResponse>| {
            let s = sender.clone();
            async move {
                if let Some(m) = s.lock().await.take() {
                    let _ = m.send(response);
                }
                Ok(())
            }
            .boxed()
        })
        .await?;

    while request.next_signal().await?.is_some() {}
    request.disconnect_signal(signal_id).await?;

    let wallpaper = receiver.await.unwrap();
    Ok(wallpaper)
}

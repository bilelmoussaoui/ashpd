//! # Examples
//!
//! Taking a screenshot
//!
//! ```rust,no_run
//! use ashpd::{desktop::screenshot, Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! async fn run() -> Result<()> {
//!     let identifier = WindowIdentifier::default();
//!     if let Ok(Response::Ok(screenshot)) = screenshot::take(identifier, true, false).await {
//!         println!("URI: {}", screenshot.uri);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! Picking a color
//! ```rust,no_run
//! use ashpd::{desktop::screenshot, Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! async fn run() -> Result<()> {
//!     let identifier = WindowIdentifier::default();
//!     if let Ok(Response::Ok(color)) = screenshot::pick_color(identifier).await {
//!         println!("({}, {}, {})", color.red(), color.green(), color.blue());
//!     }
//!     Ok(())
//! }
//! ```
use std::sync::Arc;

use futures::{lock::Mutex, FutureExt};
use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{AsyncRequestProxy, HandleToken, RequestProxy, Response, WindowIdentifier};

#[derive(SerializeDict, DeserializeDict, TypeDict, Clone, Debug, Default)]
/// Specified options on a screenshot request.
pub struct ScreenshotOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
    /// Whether the dialog should be modal.
    modal: Option<bool>,
    /// Hint whether the dialog should offer customization before taking a
    /// screenshot.
    interactive: Option<bool>,
}

impl ScreenshotOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    /// Sets whether the dialog should be a modal.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }

    /// Sets whether the dialog should offer customization before a screenshot
    /// or not.
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = Some(interactive);
        self
    }
}

#[derive(DeserializeDict, SerializeDict, Clone, TypeDict, Debug)]
/// A response to a screenshot request.
pub struct Screenshot {
    /// The screenshot uri.
    pub uri: String,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Clone, Debug, Default)]
/// Specified options on a pick color request.
pub struct PickColorOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
}

impl PickColorOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, Clone, Copy, PartialEq, TypeDict)]
/// A response to a pick color request.
/// **Note** the values are normalized.
pub struct Color {
    color: ([f64; 3]),
}

impl Color {
    /// Red.
    pub fn red(&self) -> f64 {
        self.color[0]
    }

    /// Green.
    pub fn green(&self) -> f64 {
        self.color[1]
    }

    /// Blue.
    pub fn blue(&self) -> f64 {
        self.color[2]
    }
}

#[cfg(feature = "feature_gtk3")]
impl Into<gtk3::gdk::RGBA> for Color {
    fn into(self) -> gtk3::gdk::RGBA {
        gtk3::gdk::RGBA {
            red: self.red(),
            green: self.green(),
            blue: self.blue(),
            alpha: 1_f64,
        }
    }
}

#[cfg(feature = "feature_gtk4")]
impl Into<gtk4::gdk::RGBA> for Color {
    fn into(self) -> gtk4::gdk::RGBA {
        gtk4::gdk::RGBA {
            red: self.red() as f32,
            green: self.green() as f32,
            blue: self.blue() as f32,
            alpha: 1_f32,
        }
    }
}

impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Color")
            .field("red", &self.red())
            .field("green", &self.green())
            .field("blue", &self.blue())
            .finish()
    }
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Screenshot",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications request a screenshot.
trait Screenshot {
    /// Obtains the color of a single pixel.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `options` - A [`PickColorOptions`].
    ///
    /// [`PickColorOptions`]: ./struct.PickColorOptions.html
    #[dbus_proxy(object = "Request")]
    fn pick_color(&self, parent_window: WindowIdentifier, options: PickColorOptions);

    /// Takes a screenshot.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `options` - A [`ScreenshotOptions`].
    ///
    /// [`ScreenshotOptions`]: ./struct.ScreenshotOptions.html
    #[dbus_proxy(object = "Request")]
    fn screenshot(&self, parent_window: WindowIdentifier, options: ScreenshotOptions);

    /// The version of this DBus interface.
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

/// Ask the compositor to pick a color.
///
/// A helper function around the `AsyncScreenshotProxy::pick_color`.
pub async fn pick_color(window_identifier: WindowIdentifier) -> zbus::Result<Response<Color>> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncScreenshotProxy::new(&connection)?;
    let request = proxy
        .pick_color(window_identifier, PickColorOptions::default())
        .await?;

    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Arc::new(Mutex::new(Some(sender)));
    let signal_id = request
        .connect_response(move |response: Response<Color>| {
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

    let color = receiver.await.unwrap();
    Ok(color)
}

/// Request a screenshot.
///
/// An async function around the `AsyncScreenshotProxy::screenshot`.
pub async fn take(
    window_identifier: WindowIdentifier,
    interactive: bool,
    modal: bool,
) -> zbus::Result<Response<Screenshot>> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncScreenshotProxy::new(&connection)?;
    let request = proxy
        .screenshot(
            window_identifier,
            ScreenshotOptions::default()
                .interactive(interactive)
                .modal(modal),
        )
        .await?;

    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Arc::new(Mutex::new(Some(sender)));
    let signal_id = request
        .connect_response(move |response: Response<Screenshot>| {
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

    let screenshot = receiver.await.unwrap();
    Ok(screenshot)
}

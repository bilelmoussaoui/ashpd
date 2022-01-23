//! # Examples
//!
//! ## Taking a screenshot
//!
//! ```rust,no_run
//! use ashpd::desktop::screenshot;
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let uri = screenshot::take(&WindowIdentifier::default(), true, true).await?;
//!     println!("URI: {}", uri);
//!     Ok(())
//! }
//! ```
//!
//! Or by using the Proxy directly
//!
//! ```rust,no_run
//! use ashpd::desktop::screenshot::ScreenshotProxy;
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = ScreenshotProxy::new(&connection).await?;
//!
//!     let uri = proxy.screenshot(&WindowIdentifier::default(), true, true).await?;
//!     println!("URI: {}", uri);
//!     Ok(())
//! }
//! ```
//!
//! ## Picking a color
//!
//! ```rust,no_run
//! use ashpd::desktop::screenshot;
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let color = screenshot::pick_color(&WindowIdentifier::default()).await?;
//!     println!("({}, {}, {})", color.red(), color.green(), color.blue());
//!
//!     Ok(())
//! }
//! ```
//!
//! Or by using the Proxy directly
//!
//! ```rust,no_run
//! use ashpd::desktop::screenshot::ScreenshotProxy;
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = ScreenshotProxy::new(&connection).await?;
//!
//!     let color = proxy.pick_color(&WindowIdentifier::default()).await?;
//!     println!("({}, {}, {})", color.red(), color.green(), color.blue());
//!
//!     Ok(())
//! }
//! ```

use std::fmt::Debug;

use zbus::zvariant::{DeserializeDict, SerializeDict, Type};

use super::{HandleToken, DESTINATION, PATH};
use crate::{helpers::call_request_method, Error, WindowIdentifier};

#[derive(SerializeDict, DeserializeDict, Type, Clone, Debug, Default)]
/// Specified options for a [`ScreenshotProxy::screenshot`] request.
#[zvariant(signature = "dict")]
struct ScreenshotOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// Whether the dialog should be modal.
    modal: Option<bool>,
    /// Hint whether the dialog should offer customization before taking a
    /// screenshot.
    interactive: Option<bool>,
}

impl ScreenshotOptions {
    /// Sets whether the dialog should be a modal.
    #[must_use]
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }

    /// Sets whether the dialog should offer customization before a screenshot
    /// or not.
    #[must_use]
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = Some(interactive);
        self
    }
}

#[derive(DeserializeDict, SerializeDict, Clone, Type)]
/// A response to a [`ScreenshotProxy::screenshot`] request.
#[zvariant(signature = "dict")]
struct Screenshot {
    /// The screenshot uri.
    uri: String,
}

impl Debug for Screenshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.uri)
    }
}

#[derive(SerializeDict, DeserializeDict, Type, Clone, Debug, Default)]
/// Specified options for a [`ScreenshotProxy::pick_color`] request.
#[zvariant(signature = "dict")]
struct PickColorOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
}

#[derive(SerializeDict, DeserializeDict, Clone, Copy, PartialEq, Type)]
/// A response to a [`ScreenshotProxy::pick_color`] request.
/// **Note** the values are normalized.
#[zvariant(signature = "dict")]
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
impl From<Color> for gtk3::gdk::RGBA {
    fn from(color: Color) -> Self {
        gtk3::gdk::RGBA::new(color.red(), color.green(), color.blue(), 1.0)
    }
}

#[cfg(feature = "feature_gtk4")]
impl From<Color> for gtk4::gdk::RGBA {
    fn from(color: Color) -> Self {
        gtk4::gdk::RGBA::builder()
            .red(color.red() as f32)
            .green(color.green() as f32)
            .blue(color.blue() as f32)
            .build()
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

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "({}, {}, {})",
            self.red(),
            self.green(),
            self.blue()
        ))
    }
}

/// The interface lets sandboxed applications request a screenshot.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Screenshot`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Screenshot).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Screenshot")]
pub struct ScreenshotProxy<'a>(zbus::Proxy<'a>);

impl<'a> ScreenshotProxy<'a> {
    /// Create a new instance of [`ScreenshotProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<ScreenshotProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Screenshot")?
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

    /// Obtains the color of a single pixel.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    ///
    /// # Specifications
    ///
    /// See also [`PickColor`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Screenshot.PickColor).
    #[doc(alias = "PickColor")]
    #[doc(alias = "xdp_portal_pick_color")]
    pub async fn pick_color(&self, identifier: &WindowIdentifier) -> Result<Color, Error> {
        let options = PickColorOptions::default();
        call_request_method(
            self.inner(),
            &options.handle_token,
            "PickColor",
            &(&identifier, &options),
        )
        .await
    }

    /// Takes a screenshot.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    /// * `interactive` - Sets whether the dialog should offer customization
    ///   before a screenshot or not.
    /// * `modal` - Sets whether the dialog should be a modal.
    ///
    /// # Returns
    ///
    /// The screenshot URI.
    ///
    /// # Specifications
    ///
    /// See also [`Screenshot`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Screenshot.Screenshot).
    #[doc(alias = "Screenshot")]
    #[doc(alias = "xdp_portal_take_screenshot")]
    pub async fn screenshot(
        &self,
        identifier: &WindowIdentifier,
        interactive: bool,
        modal: bool,
    ) -> Result<String, Error> {
        let options = ScreenshotOptions::default()
            .interactive(interactive)
            .modal(modal);
        let response: Screenshot = call_request_method(
            self.inner(),
            &options.handle_token,
            "Screenshot",
            &(&identifier, &options),
        )
        .await?;
        Ok(response.uri)
    }
}

#[doc(alias = "xdp_portal_pick_color")]
/// A handy wrapper around [`ScreenshotProxy::pick_color`].
pub async fn pick_color(identifier: &WindowIdentifier) -> Result<Color, Error> {
    let connection = zbus::Connection::session().await?;
    let proxy = ScreenshotProxy::new(&connection).await?;
    proxy.pick_color(identifier).await
}

#[doc(alias = "xdp_portal_take_screenshot")]
/// A handy wrapper around [`ScreenshotProxy::screenshot`].
pub async fn take(
    identifier: &WindowIdentifier,
    interactive: bool,
    modal: bool,
) -> Result<String, Error> {
    let connection = zbus::Connection::session().await?;
    let proxy = ScreenshotProxy::new(&connection).await?;
    proxy.screenshot(identifier, interactive, modal).await
}

//! # Examples
//!
//! ## Taking a screenshot
//!
//! ```rust,no_run
//! use ashpd::{
//!     desktop::screenshot::{ScreenshotOptions, ScreenshotProxy},
//!     WindowIdentifier,
//! };
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let identifier = WindowIdentifier::default();
//!     let connection = zbus::azync::Connection::new_session().await?;
//!
//!     let proxy = ScreenshotProxy::new(&connection).await?;
//!     let screenshot = proxy
//!         .screenshot(identifier, ScreenshotOptions::default().interactive(true))
//!         .await?;
//!     println!("URI: {}", screenshot.uri);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Picking a color
//!
//! ```rust,no_run
//! use ashpd::{
//!     desktop::screenshot::{PickColorOptions, ScreenshotProxy},
//!     WindowIdentifier,
//! };
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let identifier = WindowIdentifier::default();
//!     let connection = zbus::azync::Connection::new_session().await?;
//!
//!     let proxy = ScreenshotProxy::new(&connection).await?;
//!     let color = proxy
//!         .pick_color(identifier, PickColorOptions::default())
//!         .await?;
//!     println!("({}, {}, {})", color.red(), color.green(), color.blue());
//!
//!     Ok(())
//! }
//! ```

use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{
    helpers::{call_request_method, property},
    Error, HandleToken, WindowIdentifier,
};

#[derive(SerializeDict, DeserializeDict, TypeDict, Clone, Debug, Default)]
/// Specified options for a [`ScreenshotProxy::screenshot`] request.
pub struct ScreenshotOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// Whether the dialog should be modal.
    modal: Option<bool>,
    /// Hint whether the dialog should offer customization before taking a
    /// screenshot.
    interactive: Option<bool>,
}

impl ScreenshotOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = handle_token;
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
/// A response to a [`ScreenshotProxy::screenshot`] request.
pub struct Screenshot {
    /// The screenshot uri.
    pub uri: String,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Clone, Debug, Default)]
/// Specified options for a [`ScreenshotProxy::pick_color`] request.
pub struct PickColorOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
}

impl PickColorOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = handle_token;
        self
    }
}

#[derive(SerializeDict, DeserializeDict, Clone, Copy, PartialEq, TypeDict)]
/// A response to a [`ScreenshotProxy::pick_color`] request.
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
impl From<Color> for gtk3::gdk::RGBA {
    fn from(color: Color) -> gtk3::gdk::RGBA {
        gtk3::gdk::RGBA {
            red: color.red(),
            green: color.green(),
            blue: color.blue(),
            alpha: 1.0,
        }
    }
}

#[cfg(feature = "feature_gtk4")]
impl From<Color> for gtk4::gdk::RGBA {
    fn from(color: Color) -> gtk4::gdk::RGBA {
        gtk4::gdk::RGBA {
            red: color.red() as f32,
            green: color.green() as f32,
            blue: color.blue() as f32,
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

/// The interface lets sandboxed applications request a screenshot.
#[derive(Debug)]
pub struct ScreenshotProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> ScreenshotProxy<'a> {
    /// Create a new instance of [`ScreenshotProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<ScreenshotProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Screenshot")
            .path("/org/freedesktop/portal/desktop")?
            .destination("org.freedesktop.portal.Desktop")
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Obtains the color of a single pixel.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `options` - A [`PickColorOptions`].
    pub async fn pick_color(
        &self,
        parent_window: WindowIdentifier,
        options: PickColorOptions,
    ) -> Result<Color, Error> {
        call_request_method(
            &self.0,
            &options.handle_token,
            "PickColor",
            &(parent_window, &options),
        )
        .await
    }

    /// Takes a screenshot.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `options` - A [`ScreenshotOptions`].
    pub async fn screenshot(
        &self,
        parent_window: WindowIdentifier,
        options: ScreenshotOptions,
    ) -> Result<Screenshot, Error> {
        call_request_method(
            &self.0,
            &options.handle_token,
            "Screenshot",
            &(parent_window, &options),
        )
        .await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}

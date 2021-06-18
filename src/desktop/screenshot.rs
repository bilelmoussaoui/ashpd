//! # Examples
//!
//! Taking a screenshot
//!
//! ```rust,no_run
//! use ashpd::{desktop::screenshot, WindowIdentifier};
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let identifier = WindowIdentifier::default();
//!     let screenshot = screenshot::take(identifier, true, false).await?;
//!     println!("URI: {}", screenshot.uri);
//!
//!     Ok(())
//! }
//! ```
//!
//! Picking a color
//! ```rust,no_run
//! use ashpd::{desktop::screenshot, WindowIdentifier};
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let identifier = WindowIdentifier::default();
//!
//!     let color = screenshot::pick_color(identifier).await?;
//!     println!("({}, {}, {})", color.red(), color.green(), color.blue());
//!
//!     Ok(())
//! }
//! ```

use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{Error, HandleToken, RequestProxy, WindowIdentifier};

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
pub struct ScreenshotProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> ScreenshotProxy<'a> {
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
    ///
    /// [`PickColorOptions`]: ./struct.PickColorOptions.html
    pub async fn pick_color(
        &self,
        parent_window: WindowIdentifier,
        options: PickColorOptions,
    ) -> Result<RequestProxy<'_>, Error> {
        let path: zvariant::OwnedObjectPath = self
            .0
            .call_method("PickColor", &(parent_window, options))
            .await?
            .body()?;
        RequestProxy::new(self.0.connection(), path).await
    }

    /// Takes a screenshot.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `options` - A [`ScreenshotOptions`].
    ///
    /// [`ScreenshotOptions`]: ./struct.ScreenshotOptions.html
    pub async fn screenshot(
        &self,
        parent_window: WindowIdentifier,
        options: ScreenshotOptions,
    ) -> Result<RequestProxy<'_>, Error> {
        let path: zvariant::OwnedObjectPath = self
            .0
            .call_method("Screenshot", &(parent_window, options))
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

/// Ask the compositor to pick a color.
///
/// A helper function around the `ScreenshotProxy::pick_color`.
pub async fn pick_color(window_identifier: WindowIdentifier) -> Result<Color, Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = ScreenshotProxy::new(&connection).await?;
    let request = proxy
        .pick_color(window_identifier, PickColorOptions::default())
        .await?;

    let color = request.receive_response::<Color>().await?;
    Ok(color)
}

/// Request a screenshot.
///
/// An async function around the `ScreenshotProxy::screenshot`.
pub async fn take(
    window_identifier: WindowIdentifier,
    interactive: bool,
    modal: bool,
) -> Result<Screenshot, Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = ScreenshotProxy::new(&connection).await?;
    let request = proxy
        .screenshot(
            window_identifier,
            ScreenshotOptions::default()
                .interactive(interactive)
                .modal(modal),
        )
        .await?;

    let screenshot = request.receive_response::<Screenshot>().await?;
    Ok(screenshot)
}

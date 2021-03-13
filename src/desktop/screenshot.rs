//! # Examples
//!
//! Taking a screenshot
//!
//! ```rust,no_run
//! use ashpd::desktop::screenshot::{Screenshot, ScreenshotOptions, ScreenshotProxy};
//! use ashpd::{Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = ScreenshotProxy::new(&connection)?;
//!     let request = proxy.screenshot(
//!         WindowIdentifier::default(),
//!         ScreenshotOptions::default().interactive(true),
//!     )?;
//!
//!     request.connect_response(|response: Response<Screenshot>| {
//!         println!("{}", response.unwrap().uri);
//!         Ok(())
//!     })?;
//!     Ok(())
//! }
//! ```
//!
//! Picking a color
//!```no_run,rust
//! use ashpd::desktop::screenshot::{Color, PickColorOptions, ScreenshotProxy};
//! use ashpd::{Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = ScreenshotProxy::new(&connection)?;
//!
//!     let request = proxy.pick_color(WindowIdentifier::default(), PickColorOptions::default())?;
//!
//!     request.connect_response(|response: Response<Color>| {
//!         if let Response::Ok(color) = response {
//!             println!("({}, {}, {})", color.red(), color.green(), color.blue());
//!         }
//!         Ok(())
//!     })?;
//!
//!     Ok(())
//! }
//! ```
use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{AsyncRequestProxy, HandleToken, RequestProxy, WindowIdentifier};

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

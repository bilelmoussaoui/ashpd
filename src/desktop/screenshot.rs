//! # Examples
//!
//! Taking a screenshot
//!
//! ```no_run
//! use ashpd::desktop::screenshot::{Screenshot, ScreenshotOptions, ScreenshotProxy};
//! use ashpd::{RequestProxy, Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = ScreenshotProxy::new(&connection)?;
//!     let request = proxy.screenshot(
//!         WindowIdentifier::default(),
//!         ScreenshotOptions::default()
//!             .interactive(true)
//!     )?;
//!
//!     request.connect_response(|response: Response<Screenshot>| {
//!         println!("{}", response.unwrap().uri);
//!         Ok(())
//!     })?;
//!     Ok(())
//! }
//!```
//!
//! Picking a color
//!```no_run
//! use ashpd::desktop::screenshot::{Color, PickColorOptions, ScreenshotProxy};
//! use ashpd::{RequestProxy, Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = ScreenshotProxy::new(&connection)?;
//!
//!     let request = proxy.pick_color(
//!             WindowIdentifier::default(),
//!             PickColorOptions::default()
//!     )?;
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
use crate::{AsyncRequestProxy, HandleToken, RequestProxy, WindowIdentifier};
use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a screenshot request.
pub struct ScreenshotOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<HandleToken>,
    /// Whether the dialog should be modal.
    pub modal: Option<bool>,
    /// Hint whether the dialog should offer customization before taking a screenshot.
    pub interactive: Option<bool>,
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

    /// Sets whether the dialog should offer customization before a screenshot or not.
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = Some(interactive);
        self
    }
}

#[derive(DeserializeDict, SerializeDict, TypeDict, Debug)]
/// A response to a screenshot request.
pub struct Screenshot {
    /// The screenshot uri.
    pub uri: String,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a pick color request.
pub struct PickColorOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<HandleToken>,
}

impl PickColorOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// A response to a pick color request.
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
impl Into<gdk3::RGBA> for Color {
    fn into(self) -> gdk3::RGBA {
        gdk3::RGBA {
            red: self.red(),
            green: self.green(),
            blue: self.blue(),
            alpha: 1_f64,
        }
    }
}

#[cfg(feature = "feature_gtk4")]
impl Into<gdk4::RGBA> for Color {
    fn into(self) -> gdk4::RGBA {
        gdk4::RGBA {
            red: self.red() as f32,
            green: self.green() as f32,
            blue: self.blue() as f32,
            alpha: 1_f32,
        }
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
    /// Returns a [`RequestProxy`].
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `options` - A [`PickColorOptions`]
    ///
    /// [`PickColorOptions`]: ./struct.PickColorOptions.html
    /// [`RequestProxy`]: ../request/struct.RequestProxy.html
    #[dbus_proxy(object = "Request")]
    fn pick_color(&self, parent_window: WindowIdentifier, options: PickColorOptions);

    /// Takes a screenshot
    ///
    /// Returns a [`RequestProxy`].
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `options` - A [`ScreenshotOptions`]
    ///
    /// [`ScreenshotOptions`]: ./struct.ScreenshotOptions.html
    /// [`RequestProxy`]: ../request/struct.RequestProxy.html
    #[dbus_proxy(object = "Request")]
    fn screenshot(&self, parent_window: WindowIdentifier, options: ScreenshotOptions);

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

//! # Examples
//!
//! Taking a screenshot
//!
//! ```no_run
//! use libportal::desktop::screenshot::{Screenshot, ScreenshotOptions, ScreenshotProxy};
//! use libportal::{RequestProxy, Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = ScreenshotProxy::new(&connection)?;
//!     let request_handle = proxy.screenshot(
//!         WindowIdentifier::default(),
//!         ScreenshotOptions::default()
//!             .interactive(true)
//!     )?;
//!
//!     let request = RequestProxy::new(&connection, &request_handle)?;
//!     request.on_response(|response: Response<Screenshot>| {
//!         println!("{}", response.unwrap().uri);
//!     })?;
//!     Ok(())
//! }
//!```
//!
//! Picking a color
//!```no_run
//! use libportal::desktop::screenshot::{Color, PickColorOptions, ScreenshotProxy};
//! use libportal::{RequestProxy, Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! fn main() -> Result<()> {
//!    let connection = zbus::Connection::new_session()?;
//!    let proxy = ScreenshotProxy::new(&connection)?;
//!
//!    let request_handle = proxy.pick_color(
//!             WindowIdentifier::default(),
//!             PickColorOptions::default()
//!    )?;
//!
//!    let request = RequestProxy::new(&connection, &request_handle)?;
//!
//!     request.on_response(|response: Response<Color>| {
//!         if let Ok(color) = response {
//!             println!("({}, {}, {})", color.red(), color.green(), color.blue());
//!         }
//!    })?;
//!
//!    Ok(())
//!}
//! ```
use crate::{HandleToken, WindowIdentifier};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::OwnedObjectPath;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a screenshot request.
pub struct ScreenshotOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<HandleToken>,
    /// Whether the dialog should be modal.
    pub modal: Option<bool>,
    /// Hint whether the dialog should offer customization before taking a screenshot.
    pub interactive: Option<bool>,
}

impl ScreenshotOptions {
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }

    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = Some(interactive);
        self
    }
}

#[derive(DeserializeDict, SerializeDict, TypeDict, Debug)]
pub struct Screenshot {
    pub uri: String,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a pick color request.
pub struct PickColorOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<HandleToken>,
}

impl PickColorOptions {
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
pub struct Color {
    color: ([f64; 3]),
}

impl Color {
    pub fn red(&self) -> f64 {
        self.color[0]
    }

    pub fn green(&self) -> f64 {
        self.color[1]
    }

    pub fn blue(&self) -> f64 {
        self.color[2]
    }
}

#[cfg(feature = "feature_gdk")]
impl Into<gdk::RGBA> for &Color {
    fn into(self) -> gdk::RGBA {
        gdk::RGBA {
            red: self.red(),
            green: self.green(),
            blue: self.blue(),
            alpha: 1_f64,
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
    /// Returns a [`RequestProxy`] object path..
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `options` - A [`PickColorOptions`]
    ///
    /// [`PickColorOptions`]: ./struct.PickColorOptions.html
    /// [`RequestProxy`]: ../request/struct.RequestProxy.html
    fn pick_color(
        &self,
        parent_window: WindowIdentifier,
        options: PickColorOptions,
    ) -> Result<OwnedObjectPath>;

    /// Takes a screenshot
    ///
    /// Returns a [`RequestProxy`] object path.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `options` - A [`ScreenshotOptions`]
    ///
    /// [`ScreenshotOptions`]: ./struct.ScreenshotOptions.html
    /// [`RequestProxy`]: ../request/struct.RequestProxy.html
    fn screenshot(
        &self,
        parent_window: WindowIdentifier,
        options: ScreenshotOptions,
    ) -> Result<OwnedObjectPath>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

use crate::WindowIdentifier;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::OwnedObjectPath;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a screenshot request.
pub struct ScreenshotOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
    /// Whether the dialog should be modal.
    pub modal: Option<bool>,
    /// Hint whether the dialog should offer customization before taking a screenshot.
    pub interactive: Option<bool>,
}

#[derive(Debug, Default)]
pub struct ScreenshotOptionsBuilder {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
    /// Whether the dialog should be modal.
    pub modal: Option<bool>,
    /// Hint whether the dialog should offer customization before taking a screenshot.
    pub interactive: Option<bool>,
}

impl ScreenshotOptionsBuilder {
    pub fn handle_token(mut self, handle_token: &str) -> Self {
        self.handle_token = Some(handle_token.to_string());
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

    pub fn build(self) -> ScreenshotOptions {
        ScreenshotOptions {
            handle_token: self.handle_token,
            interactive: self.interactive,
            modal: self.modal,
        }
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a pick color request.
pub struct PickColorOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
}

#[derive(Default, Debug)]
pub struct PickColorOptionsBuilder {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
}

impl PickColorOptionsBuilder {
    pub fn handle_token(mut self, handle_token: &str) -> Self {
        self.handle_token = Some(handle_token.to_string());
        self
    }

    pub fn build(self) -> PickColorOptions {
        PickColorOptions {
            handle_token: self.handle_token,
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
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `options` - A [`PickColorOptions`]
    ///
    /// [`PickColorOptions`]: ./struct.PickColorOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn pick_color(
        &self,
        parent_window: WindowIdentifier,
        options: PickColorOptions,
    ) -> Result<OwnedObjectPath>;

    /// Takes a screenshot
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `options` - A [`ScreenshotOptions`]
    ///
    /// [`ScreenshotOptions`]: ./struct.ScreenshotOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn screenshot(
        &self,
        parent_window: WindowIdentifier,
        options: ScreenshotOptions,
    ) -> Result<OwnedObjectPath>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

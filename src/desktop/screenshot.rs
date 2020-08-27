use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
pub struct ScreenshotOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
    /// Whether the dialog should be modal.
    pub modal: bool,
    /// Hint whether the dialog should offer customization before taking a screenshot.
    pub interactive: bool,
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        Self {
            modal: true,
            interactive: false,
            handle_token: None,
        }
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
pub struct PickColorOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
}

impl Default for PickColorOptions {
    fn default() -> Self {
        Self { handle_token: None }
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
    fn pick_color(&self, parent_window: &str, options: PickColorOptions) -> Result<String>;

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
    fn screenshot(&self, parent_window: &str, options: ScreenshotOptions) -> Result<String>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Value;

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
    /// * `options` - A hashmap
    ///
    ///     * `handle_token` - A string that will be used as the last element of the handle. Must be a valid object path element.
    ///
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn pick_color(&self, parent_window: &str, options: HashMap<&str, Value>) -> Result<String>;

    /// Takes a screenshot
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `options` - A hashmap
    ///
    ///     * `handle_token` - A string that will be used as the last element of the handle. Must be a valid object path element.
    ///     * `modal` - Whether the dialog should be modal. Default is `true`.
    ///     * `interactive`- Hint shether the dialog should offer customization before taking a screenshot. Default is `false`
    ///
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn screenshot(&self, parent_window: &str, options: HashMap<&str, Value>) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

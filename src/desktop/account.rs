use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Value;

#[dbus_proxy(
    interface = "org.freedesktop.portal.Account",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications query basic information about the user,
/// like his name and avatar photo.
///
/// The portal backend will present the user with a dialog to confirm which (if any) information to share.
trait Account {
    /// Gets information about the user.
    ///
    /// # Arguments
    ///
    /// * `handle` - A string that will be used as the last element of the handle.
    /// * `app_id` - App id of the application
    /// * `window` - Identifier for the window
    /// * `options` - A HashMap
    ///     * `reason` - A string that can be shown in the dialog to expain why the information is needed.
    fn get_user_information(
        &self,
        handle: &str,
        app_id: &str,
        window: &str,
        options: HashMap<&str, Value>,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

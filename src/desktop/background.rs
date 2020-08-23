use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Value;

#[dbus_proxy(
    interface = "org.freedesktop.portal.Background",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications request that the application
/// is allowed to run in the background or started automatically when the user logs in.
trait Background {
    /// Requests that the application is allowed to run in the background.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `options` - A HashMap
    ///     * `handle_token` - A string that will be used as the last element of the handle.
    ///     * `reason` - User-visible reason for the request.
    ///     * `autostart` - `true` if the app also wants to be started automatically at login.
    ///     * `commandline` - `[&str]` Commandline to use when autostarting at login. If this is not specified, the Exec line from the desktop file will be used.
    ///     * `dbus-activatable` - if `true`, use D-Bus activation for autostart.
    fn request_background(
        &self,
        parent_window: &str,
        options: HashMap<&str, Value>,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

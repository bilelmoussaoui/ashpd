use std::collections::HashMap;
use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Value;

#[dbus_proxy(
    interface = "org.freedesktop.portal.OpenURI",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications open URIs
/// (e.g. a http: link to the applications homepage) under the control of the user.
trait OpenURI {
    /// Asks to open the directory containing a local file in the file browser.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `fd` - File descriptor for a file
    /// * `options` - A HashMap
    ///     * `handle_token` - A string that will be used as the last element of the handle. Must be a valid object path element.
    fn open_directory(
        &self,
        parent_window: &str,
        fd: RawFd,
        options: HashMap<&str, Value>,
    ) -> Result<String>;

    /// Asks to open a local file.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `fd` - File descriptor for the file to open
    /// * `options` - A HashMap
    ///     * `handle_token` - A string that will be used as the last element of the handle. Must be a valid object path element.
    ///     * `writable` - Whether to allow the chosen application to write to the file.
    ///                    This key only takes effect the uri points to a local file that is exported in the document portal, and the chosen application is sandboxed itself.
    ///     * `ask` - Whether to ask the user to choose an app. If this is not passed, or false, the portal may use a default or pick the last choice.
    fn open_file(
        &self,
        parent_window: &str,
        fd: RawFd,
        options: HashMap<&str, Value>,
    ) -> Result<String>;

    /// Asks to open a local file.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `uri` - The uri to open
    /// * `options` - A HashMap
    ///     * `handle_token` - A string that will be used as the last element of the handle. Must be a valid object path element.
    ///     * `writable` - Whether to allow the chosen application to write to the file.
    ///                    This key only takes effect the uri points to a local file that is exported in the document portal, and the chosen application is sandboxed itself.
    ///     * `ask` - Whether to ask the user to choose an app. If this is not passed, or false, the portal may use a default or pick the last choice.
    fn open_uri(
        &self,
        parent_window: &str,
        uri: &str,
        options: HashMap<&str, Value>,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

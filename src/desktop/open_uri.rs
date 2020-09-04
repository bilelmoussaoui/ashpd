use crate::WindowIdentifier;
use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for an open directory request.
pub struct OpenDirOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options for an open file request.
pub struct OpenFileOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
    /// Whether to allow the chosen application to write to the file.
    /// This key only takes effect the uri points to a local file that is exported in the document portal, and the chosen application is sandboxed itself.
    pub writeable: bool,
    /// Whether to ask the user to choose an app. If this is not passed, or false, the portal may use a default or pick the last choice.
    pub ask: bool,
}

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
    /// * `options` - [`OpenDirOptions`]
    ///
    /// [`OpenDirOptions`]: ./struct.OpenDirOptions.html
    fn open_directory(
        &self,
        parent_window: WindowIdentifier,
        fd: RawFd,
        options: OpenDirOptions,
    ) -> Result<String>;

    /// Asks to open a local file.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `fd` - File descriptor for the file to open
    /// * `options` - [`OpenFileOptions`]
    ///
    /// [`OpenFileOptions`]: ./struct.OpenFileOptions.html
    fn open_file(
        &self,
        parent_window: WindowIdentifier,
        fd: RawFd,
        options: OpenFileOptions,
    ) -> Result<String>;

    /// Asks to open a local file.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `uri` - The uri to open
    /// * `options` - [`OpenFileOptions`]
    ///
    /// [`OpenFileOptions`]: ./struct.OpenFileOptions.html
    fn open_uri(
        &self,
        parent_window: WindowIdentifier,
        uri: &str,
        options: OpenFileOptions,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

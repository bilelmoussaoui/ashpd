//! # Examples
//!
//! Open a file
//!
//! ```no_run
//! use ashpd::desktop::open_uri::{OpenFileOptions, OpenURIProxy};
//! use ashpd::{RequestProxy, Response, BasicResponse as Basic, WindowIdentifier};
//! use zbus::{self, fdo::Result};
//! use zvariant::Fd;
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = OpenURIProxy::new(&connection)?;
//!
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!
//!     let request_handle = proxy.open_file(
//!         WindowIdentifier::default(),
//!         Fd::from(file.as_raw_fd()),
//!         OpenFileOptions::default(),
//!     )?;
//!
//!     let request = RequestProxy::new(&connection, &request_handle)?;
//!     request.on_response(|response: Response<Basic>| {
//!         println!("{}", response.is_ok());
//!     })?;
//!
//!     Ok(())
//! }
//!```
//!
//! Open a file from a URI
//!
//! ```no_run
//! use ashpd::desktop::open_uri::{OpenFileOptions, OpenURIProxy};
//! use ashpd::{RequestProxy, Response, BasicResponse as Basic, WindowIdentifier};
//! use zbus::{self, fdo::Result};
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = OpenURIProxy::new(&connection)?;
//!
//!     let request_handle = proxy.open_uri(
//!         WindowIdentifier::default(),
//!         "file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg",
//!         OpenFileOptions::default(),
//!     )?;
//!
//!     let request = RequestProxy::new(&connection, &request_handle)?;
//!     request.on_response(|response: Response<Basic>| {
//!         println!("{}", response.is_ok());
//!     })?;
//!
//!     Ok(())
//! }
//! ```
use crate::{HandleToken, WindowIdentifier};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{Fd, OwnedObjectPath};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for an open directory request.
pub struct OpenDirOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<HandleToken>,
}

impl OpenDirOptions {
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for an open file request.
pub struct OpenFileOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<HandleToken>,
    /// Whether to allow the chosen application to write to the file.
    /// This key only takes effect the uri points to a local file that is exported in the document portal, and the chosen application is sandboxed itself.
    pub writeable: Option<bool>,
    /// Whether to ask the user to choose an app. If this is not passed, or false, the portal may use a default or pick the last choice.
    pub ask: Option<bool>,
}

impl OpenFileOptions {
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    pub fn writeable(mut self, writeable: bool) -> Self {
        self.writeable = Some(writeable);
        self
    }

    pub fn ask(mut self, ask: bool) -> Self {
        self.ask = Some(ask);
        self
    }
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
    /// Returns a [`RequestProxy`] object path.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `fd` - File descriptor for a file
    /// * `options` - [`OpenDirOptions`]
    ///
    /// [`OpenDirOptions`]: ./struct.OpenDirOptions.html
    /// [`RequestProxy`]: ../request/struct.RequestProxy.html
    fn open_directory(
        &self,
        parent_window: WindowIdentifier,
        fd: Fd,
        options: OpenDirOptions,
    ) -> Result<OwnedObjectPath>;

    /// Asks to open a local file.
    ///
    /// Returns a [`RequestProxy`] object path.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `fd` - File descriptor for the file to open
    /// * `options` - [`OpenFileOptions`]
    ///
    /// [`OpenFileOptions`]: ./struct.OpenFileOptions.html
    /// [`RequestProxy`]: ../request/struct.RequestProxy.html
    fn open_file(
        &self,
        parent_window: WindowIdentifier,
        fd: Fd,
        options: OpenFileOptions,
    ) -> Result<OwnedObjectPath>;

    /// Asks to open a local file.
    ///
    /// Returns a [`RequestProxy`] object path.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `uri` - The uri to open
    /// * `options` - [`OpenFileOptions`]
    ///
    /// [`OpenFileOptions`]: ./struct.OpenFileOptions.html
    /// [`RequestProxy`]: ../request/struct.RequestProxy.html
    #[dbus_proxy(name = "OpenURI")]
    fn open_uri(
        &self,
        parent_window: WindowIdentifier,
        uri: &str,
        options: OpenFileOptions,
    ) -> Result<OwnedObjectPath>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

//! # Examples
//!
//! ## Open a file
//!
//!```rust,no_run
//! use ashpd::desktop::open_uri;
//! use std::fs::File;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let file = File::open("/home/bilelmoussaoui/adwaita-day.jpg").unwrap();
//!     open_uri::open_file(Default::default(), &file, false, true).await?;
//!     Ok(())
//! }
//! ```
//!
//! Or by using the Proxy directly
//!
//! ```rust,no_run
//! use ashpd::desktop::open_uri::OpenURIProxy;
//! use std::fs::File;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = OpenURIProxy::new(&connection).await?;
//!
//!     proxy.open_file(Default::default(), &file, false, true).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Open a directory
//!
//!```rust,no_run
//! use ashpd::desktop::open_uri;
//! use std::fs::File;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let directory = File::open("/home/bilelmoussaoui/Downloads").unwrap();
//!     open_uri::open_directory(Default::default(), &directory).await?;
//!     Ok(())
//! }
//! ```
//!
//! Or by using the Proxy directly
//!
//! ```rust,no_run
//! use ashpd::desktop::open_uri::OpenURIProxy;
//! use std::fs::File;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let directory = File::open("/home/bilelmoussaoui/Downloads").unwrap();
//!
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = OpenURIProxy::new(&connection).await?;
//!
//!     proxy.open_directory(Default::default(), &directory).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Open a file from a URI
//!
//!
//!```rust,no_run
//! use ashpd::desktop::open_uri;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let uri = "file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg";
//!     open_uri::open_uri(Default::default(), uri, false, true).await?;
//!     Ok(())
//! }
//! ```
//!
//! Or by using the Proxy directly
//!
//! ```rust,no_run
//! use ashpd::desktop::open_uri::OpenURIProxy;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = OpenURIProxy::new(&connection).await?;
//!
//!     proxy
//!         .open_uri(
//!             Default::default(),
//!             "file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg",
//!             false,
//!             true,
//!         )
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

use super::{HandleToken, DESTINATION, PATH};
use crate::{helpers::call_basic_response_method, Error, WindowIdentifier};
use std::os::unix::prelude::AsRawFd;
use zvariant::Fd;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a [`OpenURIProxy::open_directory`] request.
struct OpenDirOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a [`OpenURIProxy::open_file`] or [`OpenURIProxy::open_uri`] request.
struct OpenFileOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// Whether to allow the chosen application to write to the file.
    /// This key only takes effect the uri points to a local file that is
    /// exported in the document portal, and the chosen application is sandboxed
    /// itself.
    writeable: Option<bool>,
    /// Whether to ask the user to choose an app. If this is not passed, or
    /// false, the portal may use a default or pick the last choice.
    ask: Option<bool>,
}

impl OpenFileOptions {
    /// Whether the file should be writeable or not.
    pub fn writeable(mut self, writeable: bool) -> Self {
        self.writeable = Some(writeable);
        self
    }

    /// Whether to always ask the user which application to use or not.
    pub fn ask(mut self, ask: bool) -> Self {
        self.ask = Some(ask);
        self
    }
}

/// The interface lets sandboxed applications open URIs
/// (e.g. a http: link to the applications homepage) under the control of the
/// user.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.OpenURI`](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html#gdbus-org.freedesktop.portal.OpenURI).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.OpenURI")]
pub struct OpenURIProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> OpenURIProxy<'a> {
    /// Create a new instance of [`OpenURIProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<OpenURIProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.OpenURI")
            .path(PATH)?
            .destination(DESTINATION)
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::azync::Proxy<'_> {
        &self.0
    }

    /// Asks to open the directory containing a local file in the file browser.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    /// * `directory` - File descriptor for a file.
    #[doc(alias = "OpenDirectory")]
    pub async fn open_directory<F>(
        &self,
        identifier: WindowIdentifier,
        directory: &F,
    ) -> Result<(), Error>
    where
        F: AsRawFd,
    {
        let options = OpenDirOptions::default();
        call_basic_response_method(
            &self.0,
            &options.handle_token,
            "OpenDirectory",
            &(identifier, Fd::from(directory.as_raw_fd()), &options),
        )
        .await
    }

    /// Asks to open a local file.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    /// * `file` - File descriptor for the file to open.
    /// * `writeable` - Whether the file should be writeable or not.
    /// * `ask` - Whether to always ask the user which application to use or not.
    #[doc(alias = "OpenFile")]
    pub async fn open_file<F>(
        &self,
        identifier: WindowIdentifier,
        file: &F,
        writeable: bool,
        ask: bool,
    ) -> Result<(), Error>
    where
        F: AsRawFd,
    {
        let options = OpenFileOptions::default().ask(ask).writeable(writeable);
        call_basic_response_method(
            &self.0,
            &options.handle_token,
            "OpenFile",
            &(identifier, Fd::from(file.as_raw_fd()), &options),
        )
        .await
    }

    /// Asks to open a local file.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    /// * `uri` - The uri to open.
    /// * `writeable` - Whether the file should be writeable or not.
    /// * `ask` - Whether to always ask the user which application to use or not.
    #[doc(alias = "OpenURI")]
    pub async fn open_uri(
        &self,
        identifier: WindowIdentifier,
        uri: &str,
        writeable: bool,
        ask: bool,
    ) -> Result<(), Error> {
        let options = OpenFileOptions::default().ask(ask).writeable(writeable);
        call_basic_response_method(
            &self.0,
            &options.handle_token,
            "OpenURI",
            &(identifier, uri, &options),
        )
        .await
    }
}

#[doc(alias = "xdp_portal_open_uri")]
/// A handy wrapper around [`OpenURIProxy::open_uri`].
pub async fn open_uri(
    identifier: WindowIdentifier,
    uri: &str,
    writeable: bool,
    ask: bool,
) -> Result<(), Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = OpenURIProxy::new(&connection).await?;
    proxy.open_uri(identifier, uri, writeable, ask).await?;
    Ok(())
}

/// A handy wrapper around [`OpenURIProxy::open_file`].
pub async fn open_file<F: AsRawFd>(
    identifier: WindowIdentifier,
    file: &F,
    writeable: bool,
    ask: bool,
) -> Result<(), Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = OpenURIProxy::new(&connection).await?;
    proxy.open_file(identifier, file, writeable, ask).await?;
    Ok(())
}

#[doc(alias = "xdp_portal_open_directory")]
/// A handy wrapper around [`OpenURIProxy::open_directory`].
pub async fn open_directory<F: AsRawFd>(
    identifier: WindowIdentifier,
    directory: &F,
) -> Result<(), Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = OpenURIProxy::new(&connection).await?;
    proxy.open_directory(identifier, directory).await?;
    Ok(())
}

//! # Examples
//!
//! Open a file
//!
//! ```rust,no_run
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//! use zvariant::Fd;
//! use ashpd::{desktop::open_uri, WindowIdentifier};
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!     let identifier = WindowIdentifier::default();
//!
//!     open_uri::open_file(identifier, Fd::from(file.as_raw_fd()), false, true).await?;
//!     Ok(())
//! }
//! ```
//!
//! Open a file from a URI
//!
//! ```rust,no_run
//! use ashpd::{desktop::open_uri, WindowIdentifier};
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     open_uri::open_uri(
//!         WindowIdentifier::default(),
//!         "file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg",
//!         false,
//!         true,
//!     ).await?;
//!
//!     Ok(())
//! }
//! ```
use std::os::unix::prelude::AsRawFd;

use serde::Serialize;
use zvariant::Type;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{BasicResponse, Error, HandleToken, RequestProxy, WindowIdentifier};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for an open directory request.
pub struct OpenDirOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
}

impl OpenDirOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for an open file request.
pub struct OpenFileOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
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
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

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
pub struct OpenURIProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> OpenURIProxy<'a> {
    pub async fn new(connection: &zbus::azync::Connection) -> Result<OpenURIProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.OpenURI")
            .path("/org/freedesktop/portal/desktop")?
            .destination("org.freedesktop.portal.Desktop")
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Asks to open the directory containing a local file in the file browser.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `directory` - File descriptor for a file.
    /// * `options` - [`OpenDirOptions`].
    ///
    /// [`OpenDirOptions`]: ./struct.OpenDirOptions.html
    pub async fn open_directory<F>(
        &self,
        parent_window: WindowIdentifier,
        directory: F,
        options: OpenDirOptions,
    ) -> Result<RequestProxy<'_>, Error>
    where
        F: AsRawFd + Serialize + Type,
    {
        let path: zvariant::OwnedObjectPath = self
            .0
            .call_method(
                "OpenDirectory",
                &(parent_window, directory.as_raw_fd(), options),
            )
            .await?
            .body()?;
        RequestProxy::new(self.0.connection(), path).await
    }

    /// Asks to open a local file.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `file` - File descriptor for the file to open.
    /// * `options` - [`OpenFileOptions`].
    ///
    /// [`OpenFileOptions`]: ./struct.OpenFileOptions.html
    pub async fn open_file<F>(
        &self,
        parent_window: WindowIdentifier,
        file: F,
        options: OpenFileOptions,
    ) -> Result<RequestProxy<'_>, Error>
    where
        F: AsRawFd + Serialize + Type,
    {
        let path: zvariant::OwnedObjectPath = self
            .0
            .call_method("OpenFile", &(parent_window, file.as_raw_fd(), options))
            .await?
            .body()?;
        RequestProxy::new(self.0.connection(), path).await
    }

    /// Asks to open a local file.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `uri` - The uri to open.
    /// * `options` - [`OpenFileOptions`].
    ///
    /// [`OpenFileOptions`]: ./struct.OpenFileOptions.html
    pub async fn open_uri(
        &self,
        parent_window: WindowIdentifier,
        uri: &str,
        options: OpenFileOptions,
    ) -> Result<RequestProxy<'_>, Error> {
        let path: zvariant::OwnedObjectPath = self
            .0
            .call_method("OpenURI", &(parent_window, uri, options))
            .await?
            .body()?;
        RequestProxy::new(self.0.connection(), path).await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        self.0
            .get_property::<u32>("version")
            .await
            .map_err(From::from)
    }
}

/// Open a URI.
///
/// A helper wrapper around `AsyncOpenUriProxy::open_uri`.
pub async fn open_uri(
    window_identifier: WindowIdentifier,
    uri: &str,
    writable: bool,
    ask: bool,
) -> Result<(), Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = OpenURIProxy::new(&connection).await?;
    let request = proxy
        .open_uri(
            window_identifier,
            uri,
            OpenFileOptions::default().writeable(writable).ask(ask),
        )
        .await?;

    let _response = request.receive_response::<BasicResponse>().await?;
    Ok(())
}

/// Open a file.
///
/// A helper wrapper around `AsyncOpenUriProxy::open_file`.
pub async fn open_file<F: AsRawFd + Serialize + Type>(
    window_identifier: WindowIdentifier,
    file: F,
    writable: bool,
    ask: bool,
) -> Result<(), Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = OpenURIProxy::new(&connection).await?;
    let request = proxy
        .open_file(
            window_identifier,
            file,
            OpenFileOptions::default().writeable(writable).ask(ask),
        )
        .await?;

    let _response = request.receive_response::<BasicResponse>().await?;
    Ok(())
}

/// Open a directory.
///
/// A helper wrapper around `AsyncOpenUriProxy::open_directory`.
pub async fn open_directory<F: AsRawFd + Serialize + Type>(
    window_identifier: WindowIdentifier,
    directory: F,
) -> Result<(), Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = OpenURIProxy::new(&connection).await?;
    let request = proxy
        .open_directory(window_identifier, directory, OpenDirOptions::default())
        .await?;

    let _response = request.receive_response::<BasicResponse>().await?;
    Ok(())
}

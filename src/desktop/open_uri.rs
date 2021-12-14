//! # Examples
//!
//! ## Open a file
//!
//!```rust,no_run
//! use ashpd::desktop::open_uri;
//! use ashpd::WindowIdentifier;
//! use std::fs::File;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let file = File::open("/home/bilelmoussaoui/adwaita-day.jpg").unwrap();
//!     open_uri::open_file(&WindowIdentifier::default(), &file, false, true).await?;
//!     Ok(())
//! }
//! ```
//!
//! Or by using the Proxy directly
//!
//! ```rust,no_run
//! use ashpd::desktop::open_uri::OpenURIProxy;
//! use ashpd::WindowIdentifier;
//! use std::fs::File;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = OpenURIProxy::new(&connection).await?;
//!
//!     proxy.open_file(&WindowIdentifier::default(), &file, false, true).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Open a directory
//!
//!```rust,no_run
//! use ashpd::desktop::open_uri;
//! use ashpd::WindowIdentifier;
//! use std::fs::File;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let directory = File::open("/home/bilelmoussaoui/Downloads").unwrap();
//!     open_uri::open_directory(&WindowIdentifier::default(), &directory).await?;
//!     Ok(())
//! }
//! ```
//!
//! Or by using the Proxy directly
//!
//! ```rust,no_run
//! use ashpd::desktop::open_uri::OpenURIProxy;
//! use ashpd::WindowIdentifier;
//! use std::fs::File;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let directory = File::open("/home/bilelmoussaoui/Downloads").unwrap();
//!
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = OpenURIProxy::new(&connection).await?;
//!
//!     proxy.open_directory(&WindowIdentifier::default(), &directory).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Open a file from a URI
//!
//!
//!```rust,no_run
//! use ashpd::desktop::open_uri;
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let uri = "file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg";
//!     open_uri::open_uri(&WindowIdentifier::default(), uri, false, true).await?;
//!     Ok(())
//! }
//! ```
//!
//! Or by using the Proxy directly
//!
//! ```rust,no_run
//! use ashpd::desktop::open_uri::OpenURIProxy;
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = OpenURIProxy::new(&connection).await?;
//!     let uri = "https://github.com/bilelmoussaoui/ashpd";
//!
//!     proxy.open_uri(&WindowIdentifier::default(), uri, false, true).await?;
//!     Ok(())
//! }
//! ```

use std::os::unix::prelude::AsRawFd;

use zvariant::Fd;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use super::{HandleToken, DESTINATION, PATH};
use crate::{helpers::call_basic_response_method, Error, WindowIdentifier};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a [`OpenURIProxy::open_directory`] request.
struct OpenDirOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    // Token to activate the choosen application.
    activation_token: Option<String>,
}

impl OpenDirOptions {
    pub fn set_activation_token(&mut self, activation_token: &str) {
        self.activation_token = Some(activation_token.to_string());
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a [`OpenURIProxy::open_file`] or
/// [`OpenURIProxy::open_uri`] request.
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
    // Token to activate the choosen application.
    activation_token: Option<String>,
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

    pub fn set_activation_token(&mut self, activation_token: &str) {
        self.activation_token = Some(activation_token.to_string());
    }
}

/// The interface lets sandboxed applications open URIs
/// (e.g. a http: link to the applications homepage) under the control of the
/// user.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.OpenURI`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.OpenURI).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.OpenURI")]
pub struct OpenURIProxy<'a>(zbus::Proxy<'a>);

impl<'a> OpenURIProxy<'a> {
    /// Create a new instance of [`OpenURIProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<OpenURIProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.OpenURI")?
            .path(PATH)?
            .destination(DESTINATION)?
            .build()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::Proxy<'_> {
        &self.0
    }

    /// Asks to open the directory containing a local file in the file browser.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    /// * `directory` - File descriptor for a file.
    /// * `activation_token` - Token used to activate the choosen application.
    ///     Available with the version 4 of the interface.
    ///
    /// # Specifications
    ///
    /// See also [`OpenDirectory`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-OpenURI.OpenDirectory).
    #[doc(alias = "OpenDirectory")]
    pub async fn open_directory<F>(
        &self,
        identifier: &WindowIdentifier,
        directory: &F,
        activation_token: Option<&str>,
    ) -> Result<(), Error>
    where
        F: AsRawFd,
    {
        let mut options = OpenDirOptions::default();
        if let Some(token) = activation_token {
            options.set_activation_token(token);
        }
        call_basic_response_method(
            &self.0,
            &options.handle_token,
            "OpenDirectory",
            &(&identifier, Fd::from(directory.as_raw_fd()), &options),
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
    /// * `ask` - Whether to always ask the user which application to use or
    ///   not.
    /// * `activation_token` - Token used to activate the choosen application.
    ///     Available with the version 4 of the interface.
    ///
    /// # Specifications
    ///
    /// See also [`OpenFile`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-OpenURI.OpenFile).
    #[doc(alias = "OpenFile")]
    pub async fn open_file<F>(
        &self,
        identifier: &WindowIdentifier,
        file: &F,
        writeable: bool,
        ask: bool,
        activation_token: Option<&str>,
    ) -> Result<(), Error>
    where
        F: AsRawFd,
    {
        let mut options = OpenFileOptions::default().ask(ask).writeable(writeable);
        if let Some(token) = activation_token {
            options.set_activation_token(token);
        }
        call_basic_response_method(
            &self.0,
            &options.handle_token,
            "OpenFile",
            &(&identifier, Fd::from(file.as_raw_fd()), &options),
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
    /// * `ask` - Whether to always ask the user which application to use or
    ///   not.
    /// * `activation_token` - Token used to activate the choosen application.
    ///     Available with the version 4 of the interface.
    ///
    /// *Note* that `file` uris are explicitly not supported by this method.
    /// Use [`Self::open_file`] or [`Self::open_directory`] instead.
    ///
    /// # Specifications
    ///
    /// See also [`OpenURI`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-OpenURI.OpenURI).
    #[doc(alias = "OpenURI")]
    pub async fn open_uri(
        &self,
        identifier: &WindowIdentifier,
        uri: &str,
        writeable: bool,
        ask: bool,
        activation_token: Option<&str>,
    ) -> Result<(), Error> {
        let mut options = OpenFileOptions::default().ask(ask).writeable(writeable);
        if let Some(token) = activation_token {
            options.set_activation_token(token);
        }
        call_basic_response_method(
            &self.0,
            &options.handle_token,
            "OpenURI",
            &(&identifier, uri, &options),
        )
        .await
    }
}

#[doc(alias = "xdp_portal_open_uri")]
/// A handy wrapper around [`OpenURIProxy::open_uri`].
pub async fn open_uri(
    identifier: &WindowIdentifier,
    uri: &str,
    writeable: bool,
    ask: bool,
    activation_token: Option<&str>,
) -> Result<(), Error> {
    let connection = zbus::Connection::session().await?;
    let proxy = OpenURIProxy::new(&connection).await?;
    proxy
        .open_uri(identifier, uri, writeable, ask, activation_token)
        .await?;
    Ok(())
}

/// A handy wrapper around [`OpenURIProxy::open_file`].
pub async fn open_file<F: AsRawFd>(
    identifier: &WindowIdentifier,
    file: &F,
    writeable: bool,
    ask: bool,
    activation_token: Option<&str>,
) -> Result<(), Error> {
    let connection = zbus::Connection::session().await?;
    let proxy = OpenURIProxy::new(&connection).await?;
    proxy
        .open_file(identifier, file, writeable, ask, activation_token)
        .await?;
    Ok(())
}

#[doc(alias = "xdp_portal_open_directory")]
/// A handy wrapper around [`OpenURIProxy::open_directory`].
pub async fn open_directory<F: AsRawFd>(
    identifier: &WindowIdentifier,
    directory: &F,
    activation_token: Option<&str>,
) -> Result<(), Error> {
    let connection = zbus::Connection::session().await?;
    let proxy = OpenURIProxy::new(&connection).await?;
    proxy
        .open_directory(identifier, directory, activation_token)
        .await?;
    Ok(())
}

//! The interface lets sandboxed applications open URIs
//! (e.g. a http: link to the applications homepage) under the control of the
//! user.
//!
//! Wrapper of the DBus interface: [`org.freedesktop.portal.OpenURI`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.OpenURI).
//!
//! # Examples
//!
//! ## Open a file
//!
//! ```rust,no_run
//! use std::fs::File;
//!
//! use ashpd::desktop::open_uri::OpenFileRequest;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let file = File::open("/home/bilelmoussaoui/adwaita-day.jpg").unwrap();
//!     OpenFileRequest::default()
//!         .ask(true)
//!         .build_file(&file)
//!         .await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Open a file from a URI
//!
//!
//! ```rust,no_run
//! use ashpd::desktop::open_uri::OpenFileRequest;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let uri =
//!         url::Url::parse("file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!     OpenFileRequest::default().ask(true).build_uri(&uri).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Open a directory
//!
//! ```rust,no_run
//! use std::fs::File;
//!
//! use ashpd::desktop::open_uri::OpenDirectoryRequest;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let directory = File::open("/home/bilelmoussaoui/Downloads").unwrap();
//!     OpenDirectoryRequest::default().build(&directory).await?;
//!     Ok(())
//! }
//! ```

use std::os::unix::prelude::AsRawFd;

use url::Url;
use zbus::zvariant::{DeserializeDict, Fd, SerializeDict, Type};

use super::{HandleToken, DESTINATION, PATH};
use crate::{
    helpers::{call_basic_response_method, session_connection},
    Error, WindowIdentifier,
};

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct OpenDirOptions {
    handle_token: HandleToken,
    activation_token: Option<String>,
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct OpenFileOptions {
    handle_token: HandleToken,
    writeable: Option<bool>,
    ask: Option<bool>,
    activation_token: Option<String>,
}

#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.OpenURI")]
struct OpenURIProxy<'a>(zbus::Proxy<'a>);

impl<'a> OpenURIProxy<'a> {
    /// Create a new instance of [`OpenURIProxy`].
    pub async fn new() -> Result<OpenURIProxy<'a>, Error> {
        let connection = session_connection().await?;
        let proxy = zbus::ProxyBuilder::new_bare(&connection)
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
    ///
    /// # Specifications
    ///
    /// See also [`OpenDirectory`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-OpenURI.OpenDirectory).
    #[doc(alias = "OpenDirectory")]
    #[doc(alias = "xdp_portal_open_directory")]
    pub async fn open_directory(
        &self,
        identifier: &WindowIdentifier,
        directory: &impl AsRawFd,
        options: OpenDirOptions,
    ) -> Result<(), Error> {
        call_basic_response_method(
            self.inner(),
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
    ///
    /// # Specifications
    ///
    /// See also [`OpenFile`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-OpenURI.OpenFile).
    #[doc(alias = "OpenFile")]
    pub async fn open_file(
        &self,
        identifier: &WindowIdentifier,
        file: &impl AsRawFd,
        options: OpenFileOptions,
    ) -> Result<(), Error> {
        call_basic_response_method(
            self.inner(),
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
    ///
    /// *Note* that `file` uris are explicitly not supported by this method.
    /// Use [`Self::open_file`] or [`Self::open_directory`] instead.
    ///
    /// # Specifications
    ///
    /// See also [`OpenURI`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-OpenURI.OpenURI).
    #[doc(alias = "OpenURI")]
    #[doc(alias = "xdp_portal_open_uri")]
    pub async fn open_uri(
        &self,
        identifier: &WindowIdentifier,
        uri: &url::Url,
        options: OpenFileOptions,
    ) -> Result<(), Error> {
        call_basic_response_method(
            self.inner(),
            &options.handle_token,
            "OpenURI",
            &(&identifier, uri, &options),
        )
        .await
    }
}

#[derive(Debug, Default)]
#[doc(alias = "xdp_portal_open_uri")]
pub struct OpenFileRequest {
    identifier: WindowIdentifier,
    writeable: Option<bool>,
    ask: Option<bool>,
}

impl OpenFileRequest {
    #[must_use]
    /// Sets a window identifier.
    pub fn identifier(mut self, identifier: WindowIdentifier) -> Self {
        self.identifier = identifier;
        self
    }

    #[must_use]
    /// Whether the file should be writeable or not.
    pub fn writeable(mut self, writeable: bool) -> Self {
        self.writeable = Some(writeable);
        self
    }

    #[must_use]
    /// Whether to always ask the user which application to use or not.
    pub fn ask(mut self, ask: bool) -> Self {
        self.ask = Some(ask);
        self
    }

    pub async fn build_file(self, file: &impl AsRawFd) -> Result<(), Error> {
        let proxy = OpenURIProxy::new().await?;
        let options = OpenFileOptions {
            writeable: self.writeable,
            ask: self.ask,
            ..Default::default()
        };
        proxy.open_file(&self.identifier, file, options).await
    }

    pub async fn build_uri(self, uri: &Url) -> Result<(), Error> {
        let proxy = OpenURIProxy::new().await?;
        let options = OpenFileOptions {
            writeable: self.writeable,
            ask: self.ask,
            ..Default::default()
        };
        proxy.open_uri(&self.identifier, uri, options).await
    }
}

#[derive(Debug, Default)]
#[doc(alias = "xdp_portal_open_directory")]
pub struct OpenDirectoryRequest {
    identifier: WindowIdentifier,
}

impl OpenDirectoryRequest {
    #[must_use]
    /// Sets a window identifier.
    pub fn identifier(mut self, identifier: WindowIdentifier) -> Self {
        self.identifier = identifier;
        self
    }

    pub async fn build(self, directory: &impl AsRawFd) -> Result<(), Error> {
        let proxy = OpenURIProxy::new().await?;
        let options = OpenDirOptions::default();
        proxy
            .open_directory(&self.identifier, directory, options)
            .await
    }
}

//! Open a URI or a directory.
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
use zbus::zvariant::{Fd, SerializeDict, Type};

use super::{HandleToken, DESTINATION, PATH};
use crate::{
    helpers::{call_basic_response_method, session_connection},
    Error, WindowIdentifier,
};

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct OpenDirOptions {
    handle_token: HandleToken,
    activation_token: Option<String>,
}

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct OpenFileOptions {
    handle_token: HandleToken,
    writeable: Option<bool>,
    ask: Option<bool>,
    activation_token: Option<String>,
}

#[derive(Debug)]
struct OpenURIProxy<'a>(zbus::Proxy<'a>);

impl<'a> OpenURIProxy<'a> {
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

    pub fn inner(&self) -> &zbus::Proxy<'_> {
        &self.0
    }

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
#[doc(alias = "org.freedesktop.portal.OpenURI")]
#[doc(alias = "xdp_portal_open_uri")]
pub struct OpenFileRequest {
    identifier: WindowIdentifier,
    options: OpenFileOptions,
}

impl OpenFileRequest {
    #[must_use]
    /// Sets a window identifier.
    pub fn identifier(mut self, identifier: impl Into<Option<WindowIdentifier>>) -> Self {
        self.identifier = identifier.into().unwrap_or_default();
        self
    }

    #[must_use]
    /// Whether the file should be writeable or not.
    pub fn writeable(mut self, writeable: impl Into<Option<bool>>) -> Self {
        self.options.writeable = writeable.into();
        self
    }

    #[must_use]
    /// Whether to always ask the user which application to use or not.
    pub fn ask(mut self, ask: impl Into<Option<bool>>) -> Self {
        self.options.ask = ask.into();
        self
    }

    pub async fn build_file(self, file: &impl AsRawFd) -> Result<(), Error> {
        let proxy = OpenURIProxy::new().await?;
        proxy.open_file(&self.identifier, file, self.options).await
    }

    pub async fn build_uri(self, uri: &Url) -> Result<(), Error> {
        let proxy = OpenURIProxy::new().await?;
        proxy.open_uri(&self.identifier, uri, self.options).await
    }
}

#[derive(Debug, Default)]
#[doc(alias = "xdp_portal_open_directory")]
#[doc(alias = "org.freedesktop.portal.OpenURI")]
pub struct OpenDirectoryRequest {
    identifier: WindowIdentifier,
    options: OpenDirOptions,
}

impl OpenDirectoryRequest {
    #[must_use]
    /// Sets a window identifier.
    pub fn identifier(mut self, identifier: impl Into<Option<WindowIdentifier>>) -> Self {
        self.identifier = identifier.into().unwrap_or_default();
        self
    }

    pub async fn build(self, directory: &impl AsRawFd) -> Result<(), Error> {
        let proxy = OpenURIProxy::new().await?;
        proxy
            .open_directory(&self.identifier, directory, self.options)
            .await
    }
}

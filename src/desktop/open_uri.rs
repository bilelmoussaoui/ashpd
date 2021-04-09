//! # Examples
//!
//! Open a file
//!
//! ```rust,no_run
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//!
//! use ashpd::{desktop::open_uri, Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! async fn run() -> Result<()> {
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!     let identifier = WindowIdentifier::default();
//!
//!     if let Ok(Response::Ok(_)) = open_uri::open_file(identifier, file.as_raw_fd(), false, true).await {
//!         // Success!
//!     }
//!     Ok(())
//! }
//! ```
//!
//! Open a file from a URI
//!
//! ```rust,no_run
//! use ashpd::{desktop::open_uri, Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! async fn run() -> Result<()> {
//!
//!     if let Ok(Response::Ok(_)) = open_uri::open_uri(
//!         WindowIdentifier::default(),
//!         "file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg",
//!         false,
//!         true,
//!     )
//!     .await
//!     {
//!         // Success!
//!     }
//!     Ok(())
//! }
//! ```
use std::os::unix::prelude::AsRawFd;
use std::sync::Arc;

use futures::{lock::Mutex, FutureExt};
use serde::Serialize;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Type;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{
    AsyncRequestProxy, BasicResponse, HandleToken, RequestProxy, Response, WindowIdentifier,
};

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

#[dbus_proxy(
    interface = "org.freedesktop.portal.OpenURI",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications open URIs
/// (e.g. a http: link to the applications homepage) under the control of the
/// user.
trait OpenURI {
    /// Asks to open the directory containing a local file in the file browser.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `directory` - File descriptor for a file.
    /// * `options` - [`OpenDirOptions`].
    ///
    /// [`OpenDirOptions`]: ./struct.OpenDirOptions.html
    #[dbus_proxy(object = "Request")]
    fn open_directory<F>(
        &self,
        parent_window: WindowIdentifier,
        directory: F,
        options: OpenDirOptions,
    ) where
        F: AsRawFd + Serialize + Type;

    /// Asks to open a local file.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `file` - File descriptor for the file to open.
    /// * `options` - [`OpenFileOptions`].
    ///
    /// [`OpenFileOptions`]: ./struct.OpenFileOptions.html
    #[dbus_proxy(object = "Request")]
    fn open_file<F>(&self, parent_window: WindowIdentifier, file: F, options: OpenFileOptions)
    where
        F: AsRawFd + Serialize + Type;

    /// Asks to open a local file.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `uri` - The uri to open.
    /// * `options` - [`OpenFileOptions`].
    ///
    /// [`OpenFileOptions`]: ./struct.OpenFileOptions.html
    #[dbus_proxy(name = "OpenURI", object = "Request")]
    fn open_uri(&self, parent_window: WindowIdentifier, uri: &str, options: OpenFileOptions);

    /// The version of this DBus interface.
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

/// Open a URI.
///
/// A helper wrapper around `AsyncOpenUriProxy::open_uri`.
pub async fn open_uri(
    window_identifier: WindowIdentifier,
    uri: &str,
    writable: bool,
    ask: bool,
) -> zbus::Result<Response<BasicResponse>> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncOpenURIProxy::new(&connection)?;
    let request = proxy
        .open_uri(
            window_identifier,
            uri,
            OpenFileOptions::default().writeable(writable).ask(ask),
        )
        .await?;

    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Arc::new(Mutex::new(Some(sender)));
    let signal_id = request
        .connect_response(move |response: Response<BasicResponse>| {
            let s = sender.clone();
            async move {
                if let Some(m) = s.lock().await.take() {
                    let _ = m.send(response);
                }
                Ok(())
            }
            .boxed()
        })
        .await?;

    while request.next_signal().await?.is_some() {}
    request.disconnect_signal(signal_id).await?;

    let response = receiver.await.unwrap();
    Ok(response)
}

/// Open a file.
///
/// A helper wrapper around `AsyncOpenUriProxy::open_file`.
pub async fn open_file<F: AsRawFd + Serialize + Type>(
    window_identifier: WindowIdentifier,
    file: F,
    writable: bool,
    ask: bool,
) -> zbus::Result<Response<BasicResponse>> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncOpenURIProxy::new(&connection)?;
    let request = proxy
        .open_file(
            window_identifier,
            file,
            OpenFileOptions::default().writeable(writable).ask(ask),
        )
        .await?;

    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Arc::new(Mutex::new(Some(sender)));
    let signal_id = request
        .connect_response(move |response: Response<BasicResponse>| {
            let s = sender.clone();
            async move {
                if let Some(m) = s.lock().await.take() {
                    let _ = m.send(response);
                }
                Ok(())
            }
            .boxed()
        })
        .await?;

    while request.next_signal().await?.is_some() {}
    request.disconnect_signal(signal_id).await?;

    let response = receiver.await.unwrap();
    Ok(response)
}

/// Open a directory.
///
/// A helper wrapper around `AsyncOpenUriProxy::open_directory`.
pub async fn open_directory<F: AsRawFd + Serialize + Type>(
    window_identifier: WindowIdentifier,
    directory: F,
) -> zbus::Result<Response<BasicResponse>> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncOpenURIProxy::new(&connection)?;
    let request = proxy
        .open_directory(window_identifier, directory, OpenDirOptions::default())
        .await?;

    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Arc::new(Mutex::new(Some(sender)));
    let signal_id = request
        .connect_response(move |response: Response<BasicResponse>| {
            let s = sender.clone();
            async move {
                if let Some(m) = s.lock().await.take() {
                    let _ = m.send(response);
                }
                Ok(())
            }
            .boxed()
        })
        .await?;

    while request.next_signal().await?.is_some() {}
    request.disconnect_signal(signal_id).await?;

    let response = receiver.await.unwrap();
    Ok(response)
}

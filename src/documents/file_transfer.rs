//! # Examples
//!
//! ```rust,no_run
//! use ashpd::documents::{FileTransferProxy};
//! use std::fs::File;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = FileTransferProxy::new(&connection).await?;
//!
//!     let key = proxy.start_transfer(true, true).await?;
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!     proxy
//!         .add_files(&key, &[&file])
//!         .await?;
//!
//!     // The files would be retrieved by another process
//!     let files = proxy.retrieve_files(&key).await?;
//!     println!("{:#?}", files);
//!
//!     proxy.stop_transfer(&key).await?;
//!
//!     Ok(())
//! }
//! ```

use std::{collections::HashMap, os::unix::prelude::AsRawFd};

use zbus::zvariant::{DeserializeDict, Fd, SerializeDict, Type, Value};

use super::{DESTINATION, PATH};
use crate::{
    helpers::{call_method, receive_signal},
    Error,
};

#[derive(SerializeDict, DeserializeDict, Debug, Type, Default)]
/// Specified options for a [`FileTransferProxy::start_transfer`] request.
#[zvariant(signature = "dict")]
struct TransferOptions {
    /// Whether to allow the chosen application to write to the files.
    writeable: Option<bool>,
    /// Whether to stop the transfer automatically after the first
    /// [`retrieve_files()`][`FileTransferProxy::retrieve_files`] call.
    #[zvariant(rename = "autostop")]
    auto_stop: Option<bool>,
}

impl TransferOptions {
    /// Sets whether the chosen application can write to the files or not.
    #[must_use]
    pub fn writeable(mut self, writeable: bool) -> Self {
        self.writeable = Some(writeable);
        self
    }

    /// Whether to stop the transfer automatically after the first
    /// [`retrieve_files()`][`FileTransferProxy::retrieve_files`] call.
    #[must_use]
    pub fn auto_stop(mut self, auto_stop: bool) -> Self {
        self.auto_stop = Some(auto_stop);
        self
    }
}

/// The interface operates as a middle-man between apps when transferring files
/// via drag-and-drop or copy-paste, taking care of the necessary exporting of
/// files in the document portal.
///
/// Toolkits are expected to use the application/vnd.portal.filetransfer
/// mimetype when using this mechanism for file exchange via copy-paste or
/// drag-and-drop.
///
/// The data that is transmitted with this mimetype should be the key returned
/// by the StartTransfer method. Upon receiving this mimetype, the target should
/// call RetrieveFiles with the key, to obtain the list of files. The portal
/// will take care of exporting files in the document store as necessary to make
/// them accessible to the target.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.FileTransfer`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.FileTransfer).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.FileTransfer")]
pub struct FileTransferProxy<'a>(zbus::Proxy<'a>);

impl<'a> FileTransferProxy<'a> {
    /// Create a new instance of [`FileTransferProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<FileTransferProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.FileTransfer")?
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

    /// Adds files to a session. This method can be called multiple times on a
    /// given session. **Note** only regular files (not directories) can be
    /// added.
    ///
    /// # Arguments
    ///
    /// * `key` - A key returned by
    ///   [`start_transfer()`][`FileTransferProxy::start_transfer`].
    /// * `fds` - A list of file descriptors of the files to register.
    ///
    /// # Specifications
    ///
    /// See also [`AddFiles`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-FileTransfer.AddFiles).
    #[doc(alias = "AddFiles")]
    pub async fn add_files(&self, key: &str, fds: &[&impl AsRawFd]) -> Result<(), Error> {
        // `options` parameter doesn't seems to be used yet
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let files: Vec<Fd> = fds.iter().map(|f| Fd::from(f.as_raw_fd())).collect();

        call_method(self.inner(), "AddFiles", &(key, files, options)).await
    }

    /// Retrieves files that were previously added to the session with
    /// [`add_files()`][`FileTransferProxy::add_files`]. The files will be
    /// exported in the document portal as-needed for the caller, and they
    /// will be writable if the owner of the session allowed it.
    ///
    /// # Arguments
    ///
    /// * `key` - A key returned by
    ///   [`start_transfer()`][`FileTransferProxy::start_transfer`].
    ///
    /// # Returns
    ///
    /// The list of file paths.
    ///
    /// # Specifications
    ///
    /// See also [`RetrieveFiles`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-FileTransfer.RetrieveFiles).
    #[doc(alias = "RetrieveFiles")]
    pub async fn retrieve_files(&self, key: &str) -> Result<Vec<String>, Error> {
        // `options` parameter doesn't seems to be used yet
        // see https://github.com/GNOME/gtk/blob/master/gdk/filetransferportal.c#L284
        let options: HashMap<&str, Value<'_>> = HashMap::new();

        call_method(self.inner(), "RetrieveFiles", &(key, options)).await
    }

    /// Starts a session for a file transfer.
    /// The caller should call [`add_files()`][`FileTransferProxy::add_files`]
    /// at least once, to add files to this session.
    ///
    /// # Arguments
    ///
    /// * `writeable` - Sets whether the chosen application can write to the
    ///   files or not.
    /// * `auto_stop` - Whether to stop the transfer automatically after the
    ///   first [`retrieve_files()`][`FileTransferProxy::retrieve_files`] call.
    ///
    /// # Returns
    ///
    /// Key that can be passed to
    /// [`retrieve_files()`][`FileTransferProxy::retrieve_files`] to obtain the
    /// files.
    ///
    /// # Specifications
    ///
    /// See also [`StartTransfer`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-FileTransfer.StartTransfer).
    pub async fn start_transfer(&self, writeable: bool, auto_stop: bool) -> Result<String, Error> {
        let options = TransferOptions::default()
            .writeable(writeable)
            .auto_stop(auto_stop);
        call_method(self.inner(), "StartTransfer", &(options)).await
    }

    /// Ends the transfer.
    /// Further calls to [`add_files()`][`FileTransferProxy::add_files`] or
    /// [`retrieve_files()`][`FileTransferProxy::retrieve_files`] for this key
    /// will return an error.
    ///
    /// # Arguments
    ///
    /// * `key` - A key returned by
    ///   [`start_transfer()`][`FileTransferProxy::start_transfer`].
    ///
    /// # Specifications
    ///
    /// See also [`StopTransfer`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-FileTransfer.StopTransfer).
    #[doc(alias = "StopTransfer")]
    pub async fn stop_transfer(&self, key: &str) -> Result<(), Error> {
        call_method(self.inner(), "StopTransfer", &(key)).await
    }

    /// Emitted when the transfer is closed.
    ///
    /// # Returns
    ///
    /// * The key returned by
    ///   [`start_transfer()`][`FileTransferProxy::start_transfer`].
    ///
    /// # Specifications
    ///
    /// See also [`TransferClosed`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-signal-org-freedesktop-portal-FileTransfer.TransferClosed).
    #[doc(alias = "TransferClosed")]
    pub async fn transfer_closed(&self) -> Result<String, Error> {
        receive_signal(self.inner(), "TransferClosed").await
    }
}

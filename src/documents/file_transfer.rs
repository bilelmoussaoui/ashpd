//! # Examples
//!
//! ```rust,no_run
//! use ashpd::documents::{FileTransferProxy};
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//! use zvariant::Fd;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = FileTransferProxy::new(&connection).await?;
//!
//!     let key = proxy.start_transfer(true, true).await?;
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!     proxy
//!         .add_files(&key, &[Fd::from(file.as_raw_fd())])
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
use std::collections::HashMap;

use crate::{helpers::call_method, Error};
use futures::prelude::stream::*;
use zvariant::{Fd, Value};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use super::{DESTINATION, PATH};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a [`FileTransferProxy::start_transfer`] request.
struct TransferOptions {
    /// Whether to allow the chosen application to write to the files.
    writeable: Option<bool>,
    /// Whether to stop the transfer automatically after the first
    /// [`FileTransferProxy::retrieve_files`] call.
    #[zvariant(rename = "autostop")]
    auto_stop: Option<bool>,
}

impl TransferOptions {
    /// Sets whether the chosen application can write to the files or not.
    pub fn writeable(mut self, writeable: bool) -> Self {
        self.writeable = Some(writeable);
        self
    }

    /// Whether to stop the transfer automatically after the first
    /// [`FileTransferProxy::retrieve_files`] call.
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
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.FileTransfer")]
pub struct FileTransferProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> FileTransferProxy<'a> {
    /// Create a new instance of [`FileTransferProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<FileTransferProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.FileTransfer")
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

    /// Adds files to a session.
    /// This method can be called multiple times on a given session.
    /// **Note** that only regular files (not directories) can be added.
    ///
    /// # Arguments
    ///
    /// * `key` - A key returned by [`FileTransferProxy::start_transfer`].
    /// * `fds` - A list of file descriptors of the files to register.
    #[doc(alias = "AddFiles")]
    pub async fn add_files(&self, key: &str, fds: &[Fd]) -> Result<(), Error> {
        // FIXME: figure out the options we can take here
        let options: HashMap<&str, Value<'_>> = HashMap::new();

        call_method(&self.0, "AddFiles", &(key, fds, options)).await
    }

    /// Retrieves files that were previously added to the session with
    /// `add_files`. The files will be exported in the document portal
    /// as-needed for the caller, and they will be writable if the owner of
    /// the session allowed it.
    ///
    /// Returns the list of file paths.
    ///
    /// # Arguments
    ///
    /// * `key` - A key returned by [`FileTransferProxy::start_transfer`].
    #[doc(alias = "RetrieveFiles")]
    pub async fn retrieve_files(&self, key: &str) -> Result<Vec<String>, Error> {
        // FIXME: figure out the options we can take here
        let options: HashMap<&str, Value<'_>> = HashMap::new();

        call_method(&self.0, "RetrieveFiles", &(key, options)).await
    }
    /// Starts a session for a file transfer.
    /// The caller should call [`FileTransferProxy::add_files`] at least once, to add files to this
    /// session.
    ///
    /// # Arguments
    ///
    /// * `writeable` - Sets whether the chosen application can write to the files or not.
    /// * `auto_stop` - Whether to stop the transfer automatically after the first
    ///             [`FileTransferProxy::retrieve_files`] call.
    ///
    /// # Returns
    ///
    /// a key that can be passed to [`FileTransferProxy::retrieve_files`] to obtain the files.
    pub async fn start_transfer(&self, writeable: bool, auto_stop: bool) -> Result<String, Error> {
        let options = TransferOptions::default()
            .writeable(writeable)
            .auto_stop(auto_stop);
        call_method(&self.0, "StartTransfer", &(options)).await
    }

    /// Ends the transfer.
    /// Further calls to [`FileTransferProxy::add_files`] or [`FileTransferProxy::retrieve_files`] for this key will
    /// return an error.
    ///
    /// # Arguments
    ///
    /// * `key` - A key returned by [`FileTransferProxy::start_transfer`].
    #[doc(alias = "StopTransfer")]
    pub async fn stop_transfer(&self, key: &str) -> Result<(), Error> {
        call_method(&self.0, "StopTransfer", &(key)).await
    }

    /// Emitted when the transfer is closed.
    ///
    /// # Returns
    ///
    /// * The key returned by [`FileTransferProxy::start_transfer`]
    #[doc(alias = "TransferClosed")]
    pub async fn transfer_closed(&self) -> Result<String, Error> {
        let mut stream = self.0.receive_signal("TransferClosed").await?;
        let message = stream.next().await.ok_or(Error::NoResponse)?;
        message.body::<String>().map_err(From::from)
    }
}

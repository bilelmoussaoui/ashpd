//! # Examples
//!
//! ```rust,no_run
//! use ashpd::documents::file_transfer::{FileTransferProxy, TransferOptions};
//! use std::collections::HashMap;
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//! use zbus::{fdo::Result, Connection};
//! use zvariant::Fd;
//!
//! fn main() -> Result<()> {
//!     let connection = Connection::new_session()?;
//!     let proxy = FileTransferProxy::new(&connection)?;
//!
//!     let key = proxy.start_transfer(TransferOptions::default().writeable(true).autostop(true))?;
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!     proxy.add_files(&key, &[Fd::from(file.as_raw_fd())], HashMap::new())?;
//!
//!     // The files would be retrieved by another process
//!     let files = proxy.retrieve_files(&key, HashMap::new())?;
//!     println!("{:#?}", files);
//!
//!     proxy.stop_transfer(&key)?;
//!
//!     Ok(())
//! }
//! ```
use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{Fd, Value};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a start transfer request.
pub struct TransferOptions {
    /// Whether to allow the chosen application to write to the files.
    writeable: Option<bool>,
    /// Whether to stop the transfer automatically after the first `retrieve_files` call.
    #[zvariant(rename = "autostop")]
    auto_stop: Option<bool>,
}

impl TransferOptions {
    /// Sets whether the chosen application can write to the files or not.
    pub fn writeable(mut self, writeable: bool) -> Self {
        self.writeable = Some(writeable);
        self
    }

    /// Whether to stop the transfer automatically after the first retrieve_files call.
    pub fn auto_stop(mut self, auto_stop: bool) -> Self {
        self.auto_stop = Some(auto_stop);
        self
    }
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.FileTransfer",
    default_service = "org.freedesktop.portal.Documents",
    default_path = "/org/freedesktop/portal/documents"
)]
/// The interface operates as a middle-man between apps when transferring files
/// via drag-and-drop or copy-paste, taking care of the necessary exporting of files
/// in the document portal.
///
/// Toolkits are expected to use the application/vnd.portal.filetransfer mimetype when
/// using this mechanism for file exchange via copy-paste or drag-and-drop.
///
/// The data that is transmitted with this mimetype should be the key returned by the StartTransfer method.
/// Upon receiving this mimetype, the target should call RetrieveFiles with the key, to obtain the list of files.
/// The portal will take care of exporting files in the document store as necessary to make them accessible to the target.
trait FileTransfer {
    /// Adds files to a session.
    /// This method can be called multiple times on a given session.
    /// Note that only regular files (not directories) can be added.
    ///
    /// # Arguments
    ///
    /// * `key` - a key returned by `start_transfer`
    /// * `fds` - a list of file descriptors of the files to register
    /// * `options` - ?
    /// FIXME: figure out the options we can take here
    fn add_files(&self, key: &str, fds: &[Fd], options: HashMap<&str, Value<'_>>) -> Result<()>;

    /// Retrieves files that were previously added to the session with `add_files`.
    /// The files will be exported in the document portal as-needed for the caller,
    /// and they will be writable if the owner of the session allowed it.
    ///
    /// Returns the list of file paths
    ///
    /// # Arguments
    ///
    /// * `key` - a key returned by `start_transfer`
    /// * `options` - ?
    /// FIXME: figure out the options we can take here
    fn retrieve_files(&self, key: &str, options: HashMap<&str, Value<'_>>) -> Result<Vec<String>>;

    /// Starts a session for a file transfer.
    /// The caller should call `add_files` at least once, to add files to this session.
    ///
    /// Returns a key that can be passed to `retrieve_files` to obtain the files.
    fn start_transfer(&self, options: TransferOptions) -> Result<String>;

    /// Ends the transfer.
    /// Further calls to `add_files` or `retrieve_files` for this key will return an error.
    ///
    /// # Arguments
    ///
    /// * `key` - A key returned by `start_transfer`
    fn stop_transfer(&self, key: &str) -> Result<()>;

    #[dbus_proxy(signal)]
    fn transfer_closed(&self, key: &str) -> Result<()>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

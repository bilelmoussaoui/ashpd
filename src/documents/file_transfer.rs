use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

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
    /// AddFiles method
    fn add_files(
        &self,
        key: &str,
        fds: &[std::os::unix::io::RawFd],
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<()>;

    /// RetrieveFiles method
    fn retrieve_files(
        &self,
        key: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<Vec<String>>;

    /// StartTransfer method
    fn start_transfer(&self, options: HashMap<&str, zvariant::Value>) -> Result<String>;

    /// StopTransfer method
    fn stop_transfer(&self, key: &str) -> Result<()>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

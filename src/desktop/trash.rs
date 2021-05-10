//! # Examples
//!
//! ```rust,no_run
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//! use ashpd::desktop::trash::{trash_file, TrashStatus};
//! use zbus::fdo::Result;
//!
//! async fn run() -> Result<()> {
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!
//!     match trash_file(file.as_raw_fd()).await? {
//!         TrashStatus::Succeeded => println!("the file was removed!"),
//!         TrashStatus::Failed => println!("oof, couldn't remove the file"),
//!     };
//!
//!     Ok(())
//! }
//! ```
use std::os::unix::io::AsRawFd;

use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Type;
use zvariant_derive::Type;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Clone, Copy, Hash, Debug, Type)]
#[repr(u32)]
/// The status of moving a file to the trash.
pub enum TrashStatus {
    /// Moving the file to the trash failed.
    Failed = 0,
    /// Moving the file to the trash succeeded
    Succeeded = 1,
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Trash",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications send files to the trashcan.
trait Trash {
    /// Sends a file to the trashcan.
    /// Applications are allowed to trash a file if they can open it in
    /// read/write mode.
    ///
    /// # Arguments
    ///
    /// * `fd` - The file descriptor.
    fn trash_file<F: AsRawFd + Type + Serialize>(&self, fd: F) -> Result<TrashStatus>;

    /// The version of this DBus interface.
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

/// Sends a file to the trash
///
/// A helper function around the `AsyncTrashProxy::trash_file`.
pub async fn trash_file<F: AsRawFd + Type + Serialize>(file: F) -> Result<TrashStatus> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncTrashProxy::new(&connection);
    proxy.trash_file(file).await
}

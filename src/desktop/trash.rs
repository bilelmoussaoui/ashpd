//! # Examples
//!
//! ```no_run
//! use zbus;
//! use zvariant::Fd;
//! use libportal::desktop::trash::{TrashProxy, TrashStatus};
//! use zbus::fdo::Result;
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//!
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = TrashProxy::new(&connection)?;
//!
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night(1).jpg").unwrap();
//!
//!     match proxy.trash_file(Fd::from(file.as_raw_fd()))? {
//!         TrashStatus::Succeeded => println!("hey world"),
//!         TrashStatus::Failed => println!("tfo"),
//!         _ => println!("something else happenned"),
//!     };
//!
//!     Ok(())
//! }
//! ```
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Fd;
use zvariant_derive::Type;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
#[non_exhaustive]
pub enum TrashStatus {
    Failed = 0,
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
    /// Applications are allowed to trash a file if they can open it in r/w mode.
    ///
    /// # Arguments
    ///
    /// * `fd` - the file descriptor
    fn trash_file(&self, fd: Fd) -> Result<TrashStatus>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

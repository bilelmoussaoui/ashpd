//! # Examples
//!
//!
//!```rust,no_run
//! use ashpd::desktop::trash;
//! use std::fs::File;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let file = File::open("/home/bilelmoussaoui/adwaita-night.jpg").unwrap();
//!     trash::trash_file(&file).await?;
//!     Ok(())
//! }
//! ```
//!
//! Or by using the Proxy directly
//!
//! ```rust,no_run
//! use ashpd::desktop::trash::TrashProxy;
//! use std::fs::File;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = TrashProxy::new(&connection).await?;
//!
//!     proxy.trash_file(&file).await?;
//!     Ok(())
//! }
//! ```

use std::os::unix::io::AsRawFd;

use serde_repr::{Deserialize_repr, Serialize_repr};
use zvariant::Fd;
use zvariant_derive::Type;

use super::{DESTINATION, PATH};
use crate::{error::PortalError, helpers::call_method, Error};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Clone, Copy, Hash, Debug, Type)]
#[repr(u32)]
/// The status of moving a file to the trash.
enum TrashStatus {
    /// Moving the file to the trash failed.
    Failed = 0,
    /// Moving the file to the trash succeeded
    Succeeded = 1,
}

/// The interface lets sandboxed applications send files to the trashcan.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Trash`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Trash).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Trash")]
pub struct TrashProxy<'a>(zbus::Proxy<'a>);

impl<'a> TrashProxy<'a> {
    /// Create a new instance of [`TrashProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<TrashProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Trash")?
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

    /// Sends a file to the trashcan.
    /// Applications are allowed to trash a file if they can open it in
    /// read/write mode.
    ///
    /// # Arguments
    ///
    /// * `fd` - The file descriptor.
    ///
    /// # Specifications
    ///
    /// See also [`TrashFile`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Trash.TrashFile).
    #[doc(alias = "TrashFile")]
    pub async fn trash_file<T>(&self, fd: &T) -> Result<(), Error>
    where
        T: AsRawFd,
    {
        let status = call_method(&self.0, "TrashFile", &(Fd::from(fd.as_raw_fd()))).await?;
        match status {
            TrashStatus::Failed => Err(Error::Portal(PortalError::Failed)),
            TrashStatus::Succeeded => Ok(()),
        }
    }
}

#[doc(alias = "xdp_portal_trash_file")]
/// A handy wrapper around [`TrashProxy::trash_file`].
pub async fn trash_file<F: AsRawFd>(fd: &F) -> Result<(), Error> {
    let connection = zbus::Connection::session().await?;
    let proxy = TrashProxy::new(&connection).await?;
    proxy.trash_file(fd).await
}

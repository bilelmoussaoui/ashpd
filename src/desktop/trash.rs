//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::trash::TrashProxy;
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//! use zvariant::Fd;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = TrashProxy::new(&connection).await?;
//!
//!     proxy.trash_file(Fd::from(file.as_raw_fd())).await?;
//!     Ok(())
//! }
//! ```

use crate::{
    helpers::{call_method, property},
    Error,
};
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::os::unix::io::AsRawFd;
use zvariant::Type;
use zvariant_derive::Type;

use super::{DESTINATION, PATH};

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
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Trash")]
pub struct TrashProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> TrashProxy<'a> {
    /// Create a new instance of [`TrashProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<TrashProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Trash")
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

    /// Sends a file to the trashcan.
    /// Applications are allowed to trash a file if they can open it in
    /// read/write mode.
    ///
    /// # Arguments
    ///
    /// * `fd` - The file descriptor.
    #[doc(alias = "TrashFile")]
    pub async fn trash_file<T>(&self, fd: T) -> Result<(), Error>
    where
        T: AsRawFd + Type + Serialize,
    {
        let status = call_method(&self.0, "TrashFile", &(fd.as_raw_fd())).await?;
        match status {
            TrashStatus::Failed => Err(Error::TrashFailed),
            TrashStatus::Succeeded => Ok(()),
        }
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}

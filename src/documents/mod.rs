//! # Examples
//!
//! ```rust,no_run
//! use ashpd::documents::{DocumentsProxy, Permission};
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = DocumentsProxy::new(&connection).await?;
//!
//!     println!("{:#?}", proxy.mount_point().await?);
//!
//!     for (doc_id, host_path) in proxy.list("org.mozilla.firefox").await? {
//!         if doc_id == "f2ee988d" {
//!             let info = proxy.info(&doc_id).await?;
//!             println!("{:#?}", info);
//!         }
//!     }
//!
//!     proxy
//!         .grant_permissions(
//!             "f2ee988d",
//!             "org.mozilla.firefox",
//!             &[Permission::GrantPermissions],
//!         )
//!         .await?;
//!     proxy
//!         .revoke_permissions("f2ee988d", "org.mozilla.firefox", &[Permission::Write])
//!         .await?;
//!
//!     proxy.delete("f2ee988d").await?;
//!
//!     Ok(())
//! }
//! ```
pub(crate) const DESTINATION: &str = "org.freedesktop.portal.Documents";
pub(crate) const PATH: &str = "/org/freedesktop/portal/documents";

use crate::{helpers::call_method, Error};
use enumflags2::BitFlags;
use serde::{de::Deserializer, Deserialize, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{collections::HashMap, os::unix::prelude::AsRawFd};
use std::{fmt::Debug, str::FromStr};
use strum_macros::{AsRefStr, EnumString, IntoStaticStr, ToString};
use zvariant::{Fd, Signature, Type};
use zvariant_derive::Type;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Copy, Clone, BitFlags, Debug, Type)]
#[repr(u32)]
///
pub enum Flags {
    /// Reuse the existing document store entry for the file.
    ReuseExisting = 1,
    /// Persistent file.
    Persistent = 2,
    /// Depends on the application needs.
    AsNeededByApp = 4,
    /// Export a directory.
    ExportDirectory = 8,
}

/// A [`HashMap`] mapping application IDs to the permissions for that application
pub type Permissions = HashMap<String, Vec<Permission>>;

#[derive(Debug, Clone, AsRefStr, EnumString, IntoStaticStr, ToString, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
/// The possible permissions to grant to a specific application for a specific
/// document.
pub enum Permission {
    /// Read access.
    Read,
    /// Write access.
    Write,
    #[strum(serialize = "grant-permissions")]
    /// The possibility to grant new permissions to the file.
    GrantPermissions,
    /// Delete access.
    Delete,
}

impl zvariant::Type for Permission {
    fn signature() -> Signature<'static> {
        String::signature()
    }
}

impl Serialize for Permission {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        String::serialize(&self.to_string(), serializer)
    }
}

impl<'de> Deserialize<'de> for Permission {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Permission::from_str(&String::deserialize(deserializer)?).expect("invalid permission"))
    }
}

/// The interface lets sandboxed applications make files from the outside world
/// available to sandboxed applications in a controlled way.
///
/// Exported files will be made accessible to the application via a fuse
/// filesystem that gets mounted at `/run/user/$UID/doc/`. The filesystem gets
/// mounted both outside and inside the sandbox, but the view inside the sandbox
/// is restricted to just those files that the application is allowed to access.
///
/// Individual files will appear at `/run/user/$UID/doc/$DOC_ID/filename`,
/// where `$DOC_ID` is the ID of the file in the document store.
/// It is returned by the [`DocumentsProxy::add`] and [`DocumentsProxy::add_named`] calls.
///
/// The permissions that the application has for a document store entry (see
/// [`DocumentsProxy::grant_permissions`]) are reflected in the POSIX mode bits in the fuse
/// filesystem.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Documents`](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html#gdbus-org.freedesktop.portal.Documents).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Documents")]
pub struct DocumentsProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> DocumentsProxy<'a> {
    /// Create a new instance of [`DocumentsProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<DocumentsProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Documents")
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

    /// Adds a file to the document store.
    /// The file is passed in the form of an open file descriptor
    /// to prove that the caller has access to the file.
    ///
    /// Returns the ID of the file in the document store.
    ///
    /// # Arguments
    ///
    /// * `o_path_fd` - Open file descriptor for the file to add.
    /// * `reuse_existing` - Whether to reuse an existing document store entry
    ///   for the file.
    /// * `persistent` - Whether to add the file only for this session or
    ///   permanently.
    #[doc(alias = "Add")]
    pub async fn add<F>(
        &self,
        o_path_fd: F,
        reuse_existing: bool,
        persistent: bool,
    ) -> Result<String, Error>
    where
        F: AsRawFd + Serialize + Type + Debug,
    {
        call_method(&self.0, "Add", &(o_path_fd, reuse_existing, persistent)).await
    }

    /// Adds multiple files to the document store.
    /// The files are passed in the form of an open file descriptor
    /// to prove that the caller has access to the file.
    ///
    /// Returns the IDs of the files in the document store along with other
    /// extra info.
    ///
    /// # Arguments
    ///
    /// * `o_path_fds` - Open file descriptors for the files to export.
    /// * `flags` - A [`Flags`].
    /// * `app_id` - An application ID, or empty string.
    /// * `permissions` - The permissions to grant.
    #[doc(alias = "AddFull")]
    pub async fn add_full(
        &self,
        o_path_fds: &[Fd],
        flags: BitFlags<Flags>,
        app_id: &str,
        permissions: &[Permission],
    ) -> Result<(Vec<String>, HashMap<String, zvariant::OwnedValue>), Error> {
        call_method(
            &self.0,
            "AddFull",
            &(o_path_fds, flags, app_id, permissions),
        )
        .await
    }

    /// Creates an entry in the document store for writing a new file.
    ///
    /// Returns the ID of the file in the document store.
    ///
    /// # Arguments
    ///
    /// * `o_path_parent_fd` - Open file descriptor for the parent directory.
    /// * `filename` - The basename for the file.
    /// * `reuse_existing` - Whether to reuse an existing document store entry
    ///   for the file.
    /// * `persistent` - Whether to add the file only for this session or
    ///   permanently.
    #[doc(alias = "AddNamed")]
    pub async fn add_named<F>(
        &self,
        o_path_parent_fd: F,
        filename: &str,
        reuse_existing: bool,
        persistent: bool,
    ) -> Result<String, Error>
    where
        F: AsRawFd + Serialize + Type + Debug,
    {
        call_method(
            &self.0,
            "AddNamed",
            &(o_path_parent_fd, filename, reuse_existing, persistent),
        )
        .await
    }

    /// Adds multiple files to the document store.
    /// The files are passed in the form of an open file descriptor
    /// to prove that the caller has access to the file.
    ///
    /// Returns the ID of the file in the document store along with other extra
    /// info.
    ///
    /// # Arguments
    ///
    /// * `o_path_fd` - Open file descriptor for the parent directory.
    /// * `filename` - The basename for the file.
    /// * `flags` - A [`Flags`].
    /// * `app_id` - An application ID, or empty string.
    /// * `permissions` - The permissions to grant.
    #[doc(alias = "AddNamedFull")]
    pub async fn add_named_full<F>(
        &self,
        o_path_fd: F,
        filename: &str,
        flags: BitFlags<Flags>,
        app_id: &str,
        permissions: &[Permission],
    ) -> Result<(String, HashMap<String, zvariant::OwnedValue>), Error>
    where
        F: AsRawFd + Serialize + Type + Debug,
    {
        call_method(
            &self.0,
            "AddNamedFull",
            &(o_path_fd, filename, flags, app_id, permissions),
        )
        .await
    }

    /// Removes an entry from the document store. The file itself is not
    /// deleted. This call is available inside the sandbox if the
    /// application has the 'delete' permission for the document.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The ID of the file in the document store.
    #[doc(alias = "Delete")]
    pub async fn delete(&self, doc_id: &str) -> Result<(), Error> {
        call_method(&self.0, "Delete", &(doc_id)).await
    }

    /// Returns the path at which the document store fuse filesystem is mounted.
    /// This will typically be /run/user/$UID/doc/.
    #[doc(alias = "GetMountPoint")]
    #[doc(alias = "get_mount_point")]
    pub async fn mount_point(&self) -> Result<String, Error> {
        call_method(&self.0, "GetMountPoint", &()).await
    }

    /// Grants access permissions for a file in the document store to an
    /// application. This call is available inside the sandbox if the
    /// application has the 'grant-permissions' permission for the document.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The ID of the file in the document store.
    /// * `app_id` - The ID of the application to which permissions are granted.
    /// * `permissions` - The permissions to grant.
    #[doc(alias = "GrantPermissions")]
    pub async fn grant_permissions(
        &self,
        doc_id: &str,
        app_id: &str,
        permissions: &[Permission],
    ) -> Result<(), Error> {
        call_method(&self.0, "GrantPermissions", &(doc_id, app_id, permissions)).await
    }

    /// Gets the filesystem path and application permissions for a document
    /// store entry.
    ///
    /// Returns the path of the file in the host filesystem along with the
    /// [`Permissions`]
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The ID of the file in the document store.
    #[doc(alias = "Info")]
    pub async fn info(&self, doc_id: &str) -> Result<(String, Permissions), Error> {
        call_method(&self.0, "Info", &(doc_id)).await
    }

    /// Lists documents in the document store for an application (or for all
    /// applications).
    ///
    /// Returns a [`HashMap`] mapping document IDs to their filesystem path on the
    /// host system
    ///
    /// # Arguments
    ///
    /// * `app-id` - The application ID, or '' to list all documents.
    #[doc(alias = "List")]
    pub async fn list(&self, app_id: &str) -> Result<HashMap<String, String>, Error> {
        call_method(&self.0, "List", &(app_id)).await
    }

    /// Looks up the document ID for a file.
    /// This call is not available inside the sandbox.
    ///
    /// Returns the ID of the file in the document store,
    /// or '' if the file is not in the document store
    ///
    /// # Arguments
    ///
    /// - `filename` - A path in the host filesystem.
    #[doc(alias = "Lookup")]
    pub async fn lookup(&self, filename: &str) -> Result<String, Error> {
        call_method(&self.0, "Lookup", &(filename)).await
    }

    /// Revokes access permissions for a file in the document store from an
    /// application. This call is available inside the sandbox if the
    /// application has the 'grant-permissions' permission for the document.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The ID of the file in the document store.
    /// * `app_id` - The ID of the application from which permissions are
    ///   revoked.
    /// * `permissions` - The permissions to revoke.
    #[doc(alias = "RevokePermissions")]
    pub async fn revoke_permissions(
        &self,
        doc_id: &str,
        app_id: &str,
        permissions: &[Permission],
    ) -> Result<(), Error> {
        call_method(&self.0, "RevokePermissions", &(doc_id, app_id, permissions)).await
    }
}

/// Interact with `org.freedesktop.portal.FileTransfer` interface.
mod file_transfer;

pub use file_transfer::FileTransferProxy;

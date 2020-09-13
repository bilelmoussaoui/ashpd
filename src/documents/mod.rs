//!
//! # Examples
//!
//! ```no_run
//! use libportal::documents::{DocumentsProxy, Permission};
//! use libportal::zbus::{fdo::Result, Connection};
//!
//! fn main() -> Result<()> {
//!     let connection = Connection::new_session()?;
//!     let proxy = DocumentsProxy::new(&connection)?;
//!
//!     println!("{:#?}", proxy.get_mount_point()?);
//!
//!     for (doc_id, host_path) in proxy.list("org.mozilla.firefox")? {
//!         if doc_id == "f2ee988d" {
//!             let info = proxy.info(&doc_id)?;
//!             println!("{:#?}", info);
//!         }
//!     }
//!
//!     proxy.grant_permissions(
//!         "f2ee988d",
//!         "org.mozilla.firefox",
//!         &[Permission::GrantPermissions],
//!     )?;
//!     proxy.revoke_permissions("f2ee988d", "org.mozilla.firefox", &[Permission::Write])?;
//!
//!     proxy.delete("f2ee988d")?;
//!
//!     Ok(())
//! }
//! ```

use crate::NString;
use enumflags2::BitFlags;
use serde::{de::Deserializer, Deserialize, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString, IntoStaticStr, ToString};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{Fd, Signature};
use zvariant_derive::Type;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Copy, Clone, BitFlags, Debug, Type)]
#[repr(u32)]
pub enum Flags {
    ReuseExisting = 1,
    Persistent = 2,
    AsNeededByApp = 4,
    ExportDirectory = 8,
}

/// A `HashMap` mapping application IDs to the permissions for that application
pub type Permissions = HashMap<String, Vec<Permission>>;

#[derive(Debug, Clone, AsRefStr, EnumString, IntoStaticStr, ToString, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
pub enum Permission {
    Read,
    Write,
    #[strum(serialize = "grant-permissions")]
    GrantPermissions,
    Delete,
}

impl zvariant::Type for Permission {
    fn signature() -> Signature<'static> {
        Signature::from_string_unchecked("s".to_string())
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

#[dbus_proxy(
    interface = "org.freedesktop.portal.Documents",
    default_service = "org.freedesktop.portal.Documents",
    default_path = "/org/freedesktop/portal/documents"
)]
/// The interface lets sandboxed applications make files from the outside world available to sandboxed applications in a controlled way.
///
/// Exported files will be made accessible to the application via a fuse filesystem
/// that gets mounted at `/run/user/$UID/doc/`. The filesystem gets mounted both outside
/// and inside the sandbox, but the view inside the sandbox is restricted to just
/// those files that the application is allowed to access.
///
/// Individual files will appear at `/run/user/$UID/doc/$DOC_ID/filename`,
/// where `$DOC_ID` is the ID of the file in the document store.
/// It is returned by the `Add()` and `AddNamed()` calls.
///
/// The permissions that the application has for a document store entry (see `GrantPermissions()`)
/// are reflected in the POSIX mode bits in the fuse filesystem.
trait Documents {
    /// Adds a file to the document store.
    /// The file is passed in the form of an open file descriptor
    /// to prove that the caller has access to the file.
    ///
    /// Returns the ID of the file in the document store.
    ///
    /// # Arguments
    ///
    /// * `o_path_fd` - open file descriptor for the file to add
    /// * `reuse_existing` - whether to reuse an existing document store entry for the file
    /// * `persistent` - whether to add the file only for this session or permanently
    fn add(&self, o_path_fd: Fd, reuse_existing: bool, persistent: bool) -> Result<String>;

    /// Adds multiple files to the document store.
    /// The files are passed in the form of an open file descriptor
    /// to prove that the caller has access to the file.
    ///
    /// Returns the IDs of the files in the document store along with other extra info.
    ///
    /// # Arguments
    ///
    /// * `o_path_fds` - open file descriptors for the files to export
    /// * `flags` - a `Flags` enum.
    /// * `app_id` - an application ID, or empty string
    /// * `permissions` - the permissions to grant, possible values are 'read', 'write', 'grant-permissions' and 'delete'
    fn add_full(
        &self,
        o_path_fds: &[Fd],
        flags: BitFlags<Flags>,
        app_id: &str,
        permissions: &[Permission],
    ) -> Result<(Vec<String>, HashMap<String, zvariant::OwnedValue>)>;

    /// Creates an entry in the document store for writing a new file.
    ///
    /// Returns the ID of the file in the document store.
    ///
    /// # Arguments
    ///
    /// * `o_path_parent_fd` - open file descriptor for the parent directory
    /// * `filename` - the basename for the file
    /// * `reuse_existing` - whether to reuse an existing document store entry for the file
    /// * `persistent` - whether to add the file only for this session or permanently
    fn add_named(
        &self,
        o_path_parent_fd: Fd,
        filename: &NString,
        reuse_existing: bool,
        persistent: bool,
    ) -> Result<String>;

    /// Adds multiple files to the document store.
    /// The files are passed in the form of an open file descriptor
    /// to prove that the caller has access to the file.
    ///
    /// Returns the ID of the file in the document store along with other extra info.
    ///
    /// # Arguments
    ///
    /// * `o_path_fd` - open file descriptor for the parent directory
    /// * `filename` - the basename for the file
    /// * `flags` - a `Flags`
    /// * `app_id` - an application ID, or empty string
    /// * `permissions` - the permissions to grant.
    fn add_named_full(
        &self,
        o_path_fd: Fd,
        filename: &NString,
        flags: BitFlags<Flags>,
        app_id: &str,
        permissions: &[Permission],
    ) -> Result<(String, HashMap<String, zvariant::OwnedValue>)>;

    /// Removes an entry from the document store. The file itself is not deleted.
    /// This call is available inside the sandbox if the application
    /// has the 'delete' permission for the document.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The ID of the file in the document store
    fn delete(&self, doc_id: &str) -> Result<()>;

    /// Returns the path at which the document store fuse filesystem is mounted.
    /// This will typically be /run/user/$UID/doc/.
    fn get_mount_point(&self) -> Result<NString>;

    /// Grants access permissions for a file in the document store to an application.
    /// This call is available inside the sandbox if the application has the 'grant-permissions' permission for the document.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - the ID of the file in the document store.
    /// * `app_id` - the ID of the application to which permissions are granted.
    /// * `permissions` - the permissions to grant.
    fn grant_permissions(
        &self,
        doc_id: &str,
        app_id: &str,
        permissions: &[Permission],
    ) -> Result<()>;

    /// Gets the filesystem path and application permissions for a document store entry.
    ///
    /// Returns the path of the file in the host filesystem along with the [`Permissions`]
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The ID of the file in the document store
    fn info(&self, doc_id: &str) -> Result<(NString, Permissions)>;

    /// Lists documents in the document store for an application (or for all applications).
    ///
    /// Returns a `HashMap` mapping document IDs to their filesystem path on the host system
    ///
    /// # Arguments
    ///
    /// * `app-id` - The application ID, or '' to list all documents
    fn list(&self, app_id: &str) -> Result<HashMap<String, NString>>;

    /// Looks up the document ID for a file.
    /// This call is not available inside the sandbox.
    ///
    /// Returns the ID of the file in the document store,
    /// or '' if the file is not in the document store
    ///
    /// # Arguments
    ///
    /// - `filename` - A path in the host filesystem
    fn lookup(&self, filename: NString) -> Result<String>;

    /// Revokes access permissions for a file in the document store from an application.
    /// This call is available inside the sandbox if the application
    /// has the 'grant-permissions' permission for the document.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The ID of the file in the document store
    /// * `app_id` - The ID of the application from which permissions are revoked
    /// * `permissions` - The permissions to revoke.
    fn revoke_permissions(
        &self,
        doc_id: &str,
        app_id: &str,
        permissions: &[Permission],
    ) -> Result<()>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

/// Interact with `org.freedesktop.portal.FileTransfer` interface.
pub mod file_transfer;

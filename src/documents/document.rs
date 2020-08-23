use std::collections::HashMap;
use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};

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
    /// Add method
    fn add(&self, o_path_fd: RawFd, reuse_existing: bool, persistent: bool) -> Result<String>;

    /// AddFull method
    fn add_full(
        &self,
        o_path_fds: &[RawFd],
        flags: u32,
        app_id: &str,
        permissions: &[&str],
    ) -> Result<(Vec<String>, HashMap<String, zvariant::OwnedValue>)>;

    /// AddNamed method
    fn add_named(
        &self,
        o_path_parent_fd: RawFd,
        filename: &[u8],
        reuse_existing: bool,
        persistent: bool,
    ) -> Result<String>;

    /// AddNamedFull method
    fn add_named_full(
        &self,
        o_path_fd: RawFd,
        filename: &[u8],
        flags: u32,
        app_id: &str,
        permissions: &[&str],
    ) -> Result<(String, HashMap<String, zvariant::OwnedValue>)>;

    /// Delete method
    fn delete(&self, doc_id: &str) -> Result<()>;

    /// GetMountPoint method
    fn get_mount_point(&self) -> Result<Vec<u8>>;

    /// GrantPermissions method
    fn grant_permissions(&self, doc_id: &str, app_id: &str, permissions: &[&str]) -> Result<()>;

    /// Info method
    fn info(&self, doc_id: &str) -> Result<(Vec<u8>, HashMap<String, Vec<String>>)>;

    /// List method
    fn list(&self, app_id: &str) -> Result<HashMap<String, Vec<u8>>>;

    /// Lookup method
    fn lookup(&self, filename: &[u8]) -> Result<String>;

    /// RevokePermissions method
    fn revoke_permissions(&self, doc_id: &str, app_id: &str, permissions: &[&str]) -> Result<()>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

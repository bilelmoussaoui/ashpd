use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Documents",
    default_service = "org.freedesktop.portal.Documents",
    default_path = "/org/freedesktop/portal/documents"
)]
trait Documents {
    /// Add method
    fn add(
        &self,
        o_path_fd: std::os::unix::io::RawFd,
        reuse_existing: bool,
        persistent: bool,
    ) -> Result<String>;

    /// AddFull method
    fn add_full(
        &self,
        o_path_fds: &[std::os::unix::io::RawFd],
        flags: u32,
        app_id: &str,
        permissions: &[&str],
    ) -> Result<(Vec<String>, HashMap<String, zvariant::OwnedValue>)>;

    /// AddNamed method
    fn add_named(
        &self,
        o_path_parent_fd: std::os::unix::io::RawFd,
        filename: &[u8],
        reuse_existing: bool,
        persistent: bool,
    ) -> Result<String>;

    /// AddNamedFull method
    fn add_named_full(
        &self,
        o_path_fd: std::os::unix::io::RawFd,
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

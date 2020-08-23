use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.ScreenCast",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications create screen cast sessions.
trait ScreenCast {
    /// CreateSession method
    fn create_session(&self, options: HashMap<&str, zvariant::Value>) -> Result<String>;

    /// OpenPipeWireRemote method
    fn open_pipe_wire_remote(
        &self,
        session_handle: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<std::os::unix::io::RawFd>;

    /// SelectSources method
    fn select_sources(
        &self,
        session_handle: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// Start method
    fn start(
        &self,
        session_handle: &str,
        parent_window: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// AvailableCursorModes property
    #[dbus_proxy(property)]
    fn available_cursor_modes(&self) -> Result<u32>;

    /// AvailableSourceTypes property
    #[dbus_proxy(property)]
    fn available_source_types(&self) -> Result<u32>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

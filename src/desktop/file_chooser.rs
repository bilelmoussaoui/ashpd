use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Value;

#[dbus_proxy(
    interface = "org.freedesktop.portal.FileChooser",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications ask the user for access to files outside the sandbox.
/// The portal backend will present the user with a file chooser dialog.
trait FileChooser {
    /// OpenFile method
    fn open_file(
        &self,
        parent_window: &str,
        title: &str,
        options: HashMap<&str, Value>,
    ) -> Result<String>;

    /// SaveFile method
    fn save_file(
        &self,
        parent_window: &str,
        title: &str,
        options: HashMap<&str, Value>,
    ) -> Result<String>;

    /// SaveFiles method
    fn save_files(
        &self,
        parent_window: &str,
        title: &str,
        options: HashMap<&str, Value>,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

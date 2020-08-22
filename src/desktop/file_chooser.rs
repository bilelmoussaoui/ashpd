use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.FileChooser",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait FileChooser {
    /// OpenFile method
    fn open_file(
        &self,
        parent_window: &str,
        title: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// SaveFile method
    fn save_file(
        &self,
        parent_window: &str,
        title: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// SaveFiles method
    fn save_files(
        &self,
        parent_window: &str,
        title: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

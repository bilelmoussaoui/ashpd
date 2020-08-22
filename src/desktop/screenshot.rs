use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Screenshot",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Screenshot {
    /// PickColor method
    fn pick_color(
        &self,
        parent_window: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// Screenshot method
    fn screenshot(
        &self,
        parent_window: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

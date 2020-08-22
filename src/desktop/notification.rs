use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Notification",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Notification {
    /// AddNotification method
    fn add_notification(
        &self,
        id: &str,
        notification: HashMap<&str, zvariant::Value>,
    ) -> Result<()>;

    /// RemoveNotification method
    fn remove_notification(&self, id: &str) -> Result<()>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

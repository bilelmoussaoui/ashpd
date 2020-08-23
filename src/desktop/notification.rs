use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Notification",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications send and withdraw notifications.
///
/// It is not possible for the application to learn if the notification was actually
/// presented to the user. Not a portal in the strict sense, since there is no user interaction.
///
/// Note that in contrast to most other portal requests, notifications are expected
/// to outlast the running application. If a user clicks on a notification after
/// the application has exited, it will get activated again.
///
/// Notifications can specify actions that can be activated by the user.
/// Actions whose name starts with 'app.' are assumed to be exported and will be activated
/// via the ActivateAction() method in the org.freedesktop.Application interface.
/// Other actions are activated by sending the
///  #org.freedeskop.portal.Notification::ActionInvoked signal to the application.
///
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

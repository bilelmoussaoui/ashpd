use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Value;

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
    /// Sends a notification.
    ///
    /// The ID can be used to later withdraw the notification.
    /// If the application reuses the same ID without withdrawing, the notification is replaced by the new one.
    ///
    /// # Arguments
    ///
    /// * `id` - Application-provided ID for this notification
    /// * `notification` - HashMap
    ///     * `title` - User-visible string to display as the title.
    ///     * `body` - User-visible string to display as the body.
    ///     * `icon` -
    ///     * `priority` - The priority for the notification. Supported values: low, normal, high, urgent.
    ///     * `default-action` - Name of an action that is exported by the application. This action will be activated when the user clicks on the notification.
    ///     * `default-action-target` - Target parameter to send along when activating the default action.
    ///     * `buttons` - Array of serialized buttons to add to the notification.
    ///         * `label` - User-visible label for the button. Mandatory.
    ///         * `action` - Name of an action that is exported by the application. The action will be activated when the user clicks on the button. Mandatory.
    ///         * `target` - Target parameter to send along when activating the action.
    fn add_notification(&self, id: &str, notification: HashMap<&str, Value>) -> Result<()>;

    /// Withdraws a notification.
    ///
    /// # Arguments
    ///
    /// * `id` - Application-provided ID for this notification
    fn remove_notification(&self, id: &str) -> Result<()>;

    // FIXME: enable once signals are in
    // fn action_invoked(&self, id: &str, action: &str, params: &[Value]);

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

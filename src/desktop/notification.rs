use zbus::{dbus_proxy, fdo::Result};
use zvariant::OwnedValue;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// A notification
pub struct Notification {
    /// User-visible string to display as the title.
    pub title: String,
    /// User-visible string to display as the body.
    pub body: Option<String>,
    // Serialized icon (e.g using gio::Icon::serialize)
    // icon: Option<String>,
    /// The priority for the notification. Supported values: low, normal, high, urgent.
    /// FIXME convert to an enum
    pub priority: Option<String>,
    /// Name of an action that is exported by the application. This action will be activated when the user clicks on the notification.
    pub default_action: Option<String>,
    /// Target parameter to send along when activating the default action.
    pub default_action_target: Option<OwnedValue>,
    /// Array of buttons to add to the notification.
    pub buttons: Vec<Button>,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// A notification button
pub struct Button {
    /// User-visible label for the button. Mandatory.
    pub label: String,
    /// Name of an action that is exported by the application. The action will be activated when the user clicks on the button.
    pub action: String,
    /// Target parameter to send along when activating the action.
    pub target: Option<OwnedValue>,
}

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
    fn add_notification(&self, id: &str, notification: Notification) -> Result<()>;

    /// Withdraws a notification.
    ///
    /// # Arguments
    ///
    /// * `id` - Application-provided ID for this notification
    fn remove_notification(&self, id: &str) -> Result<()>;

    // FIXME: enable once signals are in
    // fn action_invoked(&self, id: &str, action: &str, params: &[Value]);

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

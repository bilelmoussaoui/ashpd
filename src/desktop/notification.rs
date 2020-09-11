//! # Examples
//!
//! ```no_run
//!  use libportal::desktop::notification::{
//!     Button, Notification, NotificationProxy, Priority,
//! };
//! use std::{thread, time};
//!
//! fn main() -> zbus::fdo::Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = NotificationProxy::new(&connection)?;
//!
//!     let notification_id = "org.gnome.design.Contrast";
//!     proxy.add_notification(
//!         notification_id,
//!         Notification::new("Contrast", "close")
//!             .body("color copied to clipboard")
//!             .priority(Priority::High)
//!             .button(Button::new("Copy", "copy")),
//!     )?;
//!
//!     thread::sleep(time::Duration::from_secs(1));
//!     proxy.remove_notification(notification_id)?;
//!     Ok(())
//! }
//!```
use serde::{self, Deserialize, Serialize, Serializer};
use strum_macros::{AsRefStr, EnumString, IntoStaticStr, ToString};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{OwnedValue, Signature};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(
    Debug, Clone, Deserialize, AsRefStr, EnumString, IntoStaticStr, ToString, PartialEq, Eq,
)]
#[strum(serialize_all = "lowercase")]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
}

impl zvariant::Type for Priority {
    fn signature() -> Signature<'static> {
        String::signature()
    }
}

impl Serialize for Priority {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        String::serialize(&self.to_string(), serializer)
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// A notification
pub struct Notification {
    /// User-visible string to display as the title.
    pub title: String,
    /// User-visible string to display as the body.
    pub body: Option<String>,
    /// Serialized icon (e.g using gio::Icon::serialize).
    pub icon: Option<OwnedValue>,
    /// The priority for the notification.
    pub priority: Option<Priority>,
    /// Name of an action that is exported by the application. This action will be activated when the user clicks on the notification.
    #[zvariant(rename = "default-action")]
    pub default_action: String,
    /// Target parameter to send along when activating the default action.
    #[zvariant(rename = "default-action-target")]
    pub default_action_target: Option<OwnedValue>,
    /// Array of buttons to add to the notification.
    pub buttons: Vec<Button>,
}

impl Notification {
    pub fn new(title: &str, default_action: &str) -> Self {
        Self {
            title: title.to_string(),
            body: None,
            priority: None,
            icon: None,
            default_action: default_action.to_string(),
            default_action_target: None,
            buttons: vec![],
        }
    }

    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }

    pub fn icon(mut self, icon: OwnedValue) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn default_action_target(mut self, default_action_target: OwnedValue) -> Self {
        self.default_action_target = Some(default_action_target);
        self
    }

    pub fn button(mut self, button: Button) -> Self {
        self.buttons.push(button);
        self
    }
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

impl Button {
    pub fn new(label: &str, action: &str) -> Self {
        Self {
            label: label.to_string(),
            action: action.to_string(),
            target: None,
        }
    }

    pub fn target(mut self, target: OwnedValue) -> Self {
        self.target = Some(target);
        self
    }
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

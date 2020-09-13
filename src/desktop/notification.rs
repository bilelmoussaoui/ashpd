//! # Examples
//!
//! ```no_run
//! use ashpd::desktop::notification::{Action, Button, Notification, NotificationProxy, Priority};
//! use zbus::{self, fdo::Result};
//! use zvariant::Value;
//! use std::{thread, time};
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = NotificationProxy::new(&connection)?;
//!
//!     let notification_id = "org.gnome.design.Contrast";
//!     proxy.add_notification(
//!         notification_id,
//!         Notification::new("Contrast")
//!             .default_action("open")
//!             .default_action_target(Value::U32(100).into())
//!             .body("color copied to clipboard")
//!             .priority(Priority::High)
//!             .button(Button::new("Copy", "copy").target(Value::U32(32).into()))
//!             .button(Button::new("Delete", "delete").target(Value::U32(40).into())),
//!     )?;
//!
//!     proxy.on_action_invoked(|action: Action| {
//!         match action.name() {
//!             "copy" => (),   // Copy something to clipboard
//!             "delete" => (), // Delete the file
//!             _ => (),
//!         };
//!         println!("{:#?}", action.id());
//!         println!(
//!             "{:#?}",
//!             action.parameter().get(0).unwrap().downcast_ref::<u32>()
//!         )
//!     })?;
//!
//!     thread::sleep(time::Duration::from_secs(1));
//!     proxy.remove_notification(notification_id)?;
//!     Ok(())
//! }
//!
//!```
use serde::{self, Deserialize, Serialize, Serializer};
use strum_macros::{AsRefStr, EnumString, IntoStaticStr, ToString};
use zbus::{fdo::Result, Connection, Proxy};
use zvariant::{OwnedValue, Signature};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

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
    pub default_action: Option<String>,
    /// Target parameter to send along when activating the default action.
    #[zvariant(rename = "default-action-target")]
    pub default_action_target: Option<OwnedValue>,
    /// Array of buttons to add to the notification.
    pub buttons: Option<Vec<Button>>,
}

impl Notification {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            body: None,
            priority: None,
            icon: None,
            default_action: None,
            default_action_target: None,
            buttons: None,
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

    pub fn default_action(mut self, default_action: &str) -> Self {
        self.default_action = Some(default_action.to_string());
        self
    }

    pub fn default_action_target(mut self, default_action_target: OwnedValue) -> Self {
        self.default_action_target = Some(default_action_target);
        self
    }

    pub fn button(mut self, button: Button) -> Self {
        match self.buttons {
            Some(ref mut buttons) => buttons.push(button),
            None => {
                self.buttons.replace(vec![button]);
            }
        };
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

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct Action<'a>(&'a str, &'a str, Vec<OwnedValue>);

impl<'a> Action<'a> {
    /// Notification ID.
    pub fn id(&self) -> &'a str {
        self.0
    }

    /// Action name.
    pub fn name(&self) -> &'a str {
        self.1
    }

    /// The parameters passed to the action.
    pub fn parameter(&self) -> Vec<OwnedValue> {
        self.2.clone()
    }
}

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
pub struct NotificationProxy<'a> {
    proxy: Proxy<'a>,
    connection: &'a Connection,
}

impl<'a> NotificationProxy<'a> {
    pub fn new(connection: &'a Connection) -> Result<Self> {
        let proxy = Proxy::new(
            connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Notification",
        )?;
        Ok(Self { proxy, connection })
    }

    /// Signal emitted when a particular action is invoked
    pub fn on_action_invoked<F>(&self, callback: F) -> Result<()>
    where
        F: FnOnce(Action),
    {
        loop {
            let msg = self.connection.receive_message()?;
            let msg_header = msg.header()?;
            if msg_header.message_type()? == zbus::MessageType::Signal
                && msg_header.member()? == Some("ActionInvoked")
            {
                let response = msg.body::<Action>()?;
                callback(response);
                break;
            }
        }
        Ok(())
    }

    /// Sends a notification.
    ///
    /// The ID can be used to later withdraw the notification.
    /// If the application reuses the same ID without withdrawing, the notification is replaced by the new one.
    ///
    /// # Arguments
    ///
    /// * `id` - Application-provided ID for this notification
    /// * `notification` - HashMap
    pub fn add_notification(&self, id: &str, notification: Notification) -> Result<()> {
        self.proxy.call("AddNotification", &(id, notification))?;
        Ok(())
    }

    /// Withdraws a notification.
    ///
    /// # Arguments
    ///
    /// * `id` - Application-provided ID for this notification
    pub fn remove_notification(&self, id: &str) -> Result<()> {
        self.proxy.call("RemoveNotification", &(id))?;
        Ok(())
    }

    /// version property
    pub fn version(&self) -> Result<u32> {
        self.proxy.get_property::<u32>("version")
    }
}

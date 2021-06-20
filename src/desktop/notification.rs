//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::notification::{Action, Button, Notification, NotificationProxy, Priority};
//! use std::{thread, time};
//! use zvariant::Value;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = NotificationProxy::new(&connection).await?;
//!
//!     let notification_id = "org.gnome.design.Contrast";
//!     proxy
//!         .add_notification(
//!             notification_id,
//!             Notification::new("Contrast")
//!                 .default_action("open")
//!                 .default_action_target(Value::U32(100).into())
//!                 .body("color copied to clipboard")
//!                 .priority(Priority::High)
//!                 .button(Button::new("Copy", "copy").target(Value::U32(32).into()))
//!                 .button(Button::new("Delete", "delete").target(Value::U32(40).into())),
//!         )
//!         .await?;
//!
//!     let action = proxy.receive_action_invoked().await?;
//!     match action.name() {
//!         "copy" => (),   // Copy something to clipboard
//!         "delete" => (), // Delete the file
//!         _ => (),
//!     };
//!     println!("{:#?}", action.id());
//!     println!(
//!         "{:#?}",
//!         action.parameter().get(0).unwrap().downcast_ref::<u32>()
//!     );
//!
//!     proxy.remove_notification(notification_id).await?;
//!     Ok(())
//! }
//! ```

use crate::{
    helpers::{call_method, property},
    Error,
};
use futures::prelude::stream::*;
use serde::{self, Deserialize, Serialize, Serializer};
use strum_macros::{AsRefStr, EnumString, IntoStaticStr, ToString};
use zvariant::{OwnedValue, Signature};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

use super::{DESTINATION, PATH};

#[derive(
    Debug, Clone, Deserialize, AsRefStr, EnumString, IntoStaticStr, ToString, PartialEq, Eq,
)]
#[strum(serialize_all = "lowercase")]
/// The notification priority
pub enum Priority {
    /// Low.
    Low,
    /// Normal.
    Normal,
    /// High.
    High,
    /// Urgent.
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
    title: String,
    /// User-visible string to display as the body.
    body: Option<String>,
    /// Serialized icon (e.g using gio::Icon::serialize).
    icon: Option<OwnedValue>,
    /// The priority for the notification.
    priority: Option<Priority>,
    /// Name of an action that is exported by the application.
    /// This action will be activated when the user clicks on the notification.
    #[zvariant(rename = "default-action")]
    default_action: Option<String>,
    /// Target parameter to send along when activating the default action.
    #[zvariant(rename = "default-action-target")]
    default_action_target: Option<OwnedValue>,
    /// Array of buttons to add to the notification.
    buttons: Option<Vec<Button>>,
}

impl Notification {
    /// Create a new notification.
    ///
    /// # Arguments
    ///
    /// * `title` - the notification title.
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

    /// Sets the notification body.
    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }

    /// Sets an icon to the notification.
    pub fn icon(mut self, icon: OwnedValue) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the notification priority.
    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Sets the default action when the user clicks on the notification.
    pub fn default_action(mut self, default_action: &str) -> Self {
        self.default_action = Some(default_action.to_string());
        self
    }

    /// Sets a value to be sent in the `action_invoked` signal.
    pub fn default_action_target(mut self, default_action_target: OwnedValue) -> Self {
        self.default_action_target = Some(default_action_target);
        self
    }

    /// Adds a new button to the notification.
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
    label: String,
    /// Name of an action that is exported by the application. The action will
    /// be activated when the user clicks on the button.
    action: String,
    /// Target parameter to send along when activating the action.
    target: Option<OwnedValue>,
}

impl Button {
    /// Create a new notification button.
    ///
    /// # Arguments
    ///
    /// * `label` - the user visible label of the button.
    /// * `action` - the action name to be invoked when the user clicks on the
    ///   button.
    pub fn new(label: &str, action: &str) -> Self {
        Self {
            label: label.to_string(),
            action: action.to_string(),
            target: None,
        }
    }

    /// The value to send with the action name when the button is clicked.
    pub fn target(mut self, target: OwnedValue) -> Self {
        self.target = Some(target);
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Type)]
/// An invoked action.
pub struct Action(String, String, Vec<OwnedValue>);

impl Action {
    /// Notification ID.
    pub fn id(&self) -> &str {
        &self.0
    }

    /// Action name.
    pub fn name(&self) -> &str {
        &self.1
    }

    /// The parameters passed to the action.
    pub fn parameter(&self) -> &Vec<OwnedValue> {
        &self.2
    }
}

/// The interface lets sandboxed applications send and withdraw notifications.
///
/// It is not possible for the application to learn if the notification was
/// actually presented to the user. Not a portal in the strict sense, since
/// there is no user interaction.
///
/// **Note** that in contrast to most other portal requests, notifications are
/// expected to outlast the running application. If a user clicks on a
/// notification after the application has exited, it will get activated again.
///
/// Notifications can specify actions that can be activated by the user.
/// Actions whose name starts with 'app.' are assumed to be exported and will be
/// activated via the ActivateAction() method in the org.freedesktop.Application
/// interface. Other actions are activated by sending the
///  `#org.freedeskop.portal.Notification::ActionInvoked` signal to the
/// application.
#[derive(Debug)]
pub struct NotificationProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> NotificationProxy<'a> {
    /// Create a new instance of [`NotificationProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<NotificationProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Notification")
            .path(PATH)?
            .destination(DESTINATION)
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Signal emitted when a particular action is invoked.
    pub async fn receive_action_invoked(&self) -> Result<Action, Error> {
        let mut stream = self.0.receive_signal("ActionInvoked").await?;
        let message = stream.next().await.ok_or(Error::NoResponse)?;
        message.body::<Action>().map_err(From::from)
    }

    /// Sends a notification.
    ///
    /// The ID can be used to later withdraw the notification.
    /// If the application reuses the same ID without withdrawing, the
    /// notification is replaced by the new one.
    ///
    /// # Arguments
    ///
    /// * `id` - Application-provided ID for this notification.
    /// * `notification` - The notification.
    pub async fn add_notification(
        &self,
        id: &str,
        notification: Notification,
    ) -> Result<(), Error> {
        call_method(&self.0, "AddNotification", &(id, notification)).await
    }

    /// Withdraws a notification.
    ///
    /// # Arguments
    ///
    /// * `id` - Application-provided ID for this notification.
    pub async fn remove_notification(&self, id: &str) -> Result<(), Error> {
        call_method(&self.0, "RemoveNotification", &(id)).await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}

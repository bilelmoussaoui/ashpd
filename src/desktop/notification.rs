//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::{Icon, notification::{Action, Button, Notification, NotificationProxy, Priority}};
//! use std::{thread, time};
//! use zbus::zvariant::Value;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = NotificationProxy::new(&connection).await?;
//!
//!     let notification_id = "org.gnome.design.Contrast";
//!     proxy
//!         .add_notification(
//!             notification_id,
//!             Notification::new("Contrast")
//!                 .default_action("open")
//!                 .default_action_target(Value::U32(100))
//!                 .icon(Icon::Uri("file:///home/some_user/some_dir/some_icon.png"))
//!                 .body("color copied to clipboard")
//!                 .priority(Priority::High)
//!                 .button(Button::new("Copy", "copy").target(Value::U32(32)))
//!                 .button(Button::new("Delete", "delete").target(Value::U32(40))),
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

use std::{fmt, str::FromStr};

use serde::{self, Deserialize, Serialize, Serializer};
use zbus::zvariant::{OwnedValue, SerializeDict, Signature, Type, Value};

use super::{Icon, DESTINATION, PATH};
use crate::{
    helpers::{call_method, receive_signal},
    Error,
};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
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

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Normal => write!(f, "Normal"),
            Self::High => write!(f, "High"),
            Self::Urgent => write!(f, "Urgent"),
        }
    }
}

impl AsRef<str> for Priority {
    fn as_ref(&self) -> &str {
        match self {
            Self::Low => "Low",
            Self::Normal => "Normal",
            Self::High => "High",
            Self::Urgent => "Urgent",
        }
    }
}

impl From<Priority> for &'static str {
    fn from(d: Priority) -> Self {
        match d {
            Priority::Low => "Low",
            Priority::Normal => "Normal",
            Priority::High => "High",
            Priority::Urgent => "Urgent",
        }
    }
}

impl FromStr for Priority {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Low" | "low" => Ok(Priority::Low),
            "Normal" | "normal" => Ok(Priority::Normal),
            "High" | "high" => Ok(Priority::High),
            "Urgent" | "urgent" => Ok(Priority::Urgent),
            _ => Err(Error::ParseError("Failed to parse priority, invalid value")),
        }
    }
}

impl Serialize for Priority {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string().to_lowercase())
    }
}

impl Type for Priority {
    fn signature() -> Signature<'static> {
        String::signature()
    }
}

#[derive(SerializeDict, Type, Debug)]
/// A notification
#[zvariant(signature = "dict")]
pub struct Notification<'a> {
    /// User-visible string to display as the title.
    title: &'a str,
    /// User-visible string to display as the body.
    body: Option<&'a str>,
    /// Serialized icon (e.g using gio::Icon::serialize).
    icon: Option<Icon<'a>>,
    /// The priority for the notification.
    priority: Option<Priority>,
    /// Name of an action that is exported by the application.
    /// This action will be activated when the user clicks on the notification.
    #[zvariant(rename = "default-action")]
    default_action: Option<&'a str>,
    /// Target parameter to send along when activating the default action.
    #[zvariant(rename = "default-action-target")]
    default_action_target: Option<Value<'a>>,
    /// Array of buttons to add to the notification.
    buttons: Option<Vec<Button<'a>>>,
}

impl<'a> Notification<'a> {
    /// Create a new notification.
    ///
    /// # Arguments
    ///
    /// * `title` - the notification title.
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            body: None,
            priority: None,
            icon: None,
            default_action: None,
            default_action_target: None,
            buttons: None,
        }
    }

    /// Sets the notification body.
    #[must_use]
    pub fn body(mut self, body: &'a str) -> Self {
        self.body = Some(body);
        self
    }

    /// Sets an icon to the notification.
    #[must_use]
    pub fn icon(mut self, icon: Icon<'a>) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the notification priority.
    #[must_use]
    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Sets the default action when the user clicks on the notification.
    #[must_use]
    pub fn default_action(mut self, default_action: &'a str) -> Self {
        self.default_action = Some(default_action);
        self
    }

    /// Sets a value to be sent in the `action_invoked` signal.
    #[must_use]
    pub fn default_action_target(mut self, default_action_target: Value<'a>) -> Self {
        self.default_action_target = Some(default_action_target);
        self
    }

    /// Adds a new button to the notification.
    #[must_use]
    pub fn button(mut self, button: Button<'a>) -> Self {
        match self.buttons {
            Some(ref mut buttons) => buttons.push(button),
            None => {
                self.buttons.replace(vec![button]);
            }
        };
        self
    }
}

#[derive(SerializeDict, Type, Debug)]
/// A notification button
#[zvariant(signature = "dict")]
pub struct Button<'a> {
    /// User-visible label for the button. Mandatory.
    label: &'a str,
    /// Name of an action that is exported by the application. The action will
    /// be activated when the user clicks on the button.
    action: &'a str,
    /// Target parameter to send along when activating the action.
    target: Option<Value<'a>>,
}

impl<'a> Button<'a> {
    /// Create a new notification button.
    ///
    /// # Arguments
    ///
    /// * `label` - the user visible label of the button.
    /// * `action` - the action name to be invoked when the user clicks on the
    ///   button.
    pub fn new(label: &'a str, action: &'a str) -> Self {
        Self {
            label,
            action,
            target: None,
        }
    }

    /// The value to send with the action name when the button is clicked.
    #[must_use]
    pub fn target(mut self, target: Value<'a>) -> Self {
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
    pub fn parameter(&self) -> &[OwnedValue] {
        &self.2
    }
}

/// The interface lets sandboxed applications send and withdraw notifications.
///
/// It is not possible for the application to learn if the notification was
/// actually presented to the user. Not a portal in the strict sense, since
/// there is no user interaction.
///
/// **Note** in contrast to most other portal requests, notifications are
/// expected to outlast the running application. If a user clicks on a
/// notification after the application has exited, it will get activated again.
///
/// Notifications can specify actions that can be activated by the user.
/// Actions whose name starts with 'app.' are assumed to be exported and will be
/// activated via the ActivateAction() method in the org.freedesktop.Application
/// interface. Other actions are activated by sending the
///  `#org.freedeskop.portal.Notification::ActionInvoked` signal to the
/// application.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Notification`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Notification).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Notification")]
pub struct NotificationProxy<'a>(zbus::Proxy<'a>);

impl<'a> NotificationProxy<'a> {
    /// Create a new instance of [`NotificationProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<NotificationProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Notification")?
            .path(PATH)?
            .destination(DESTINATION)?
            .build()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::Proxy<'_> {
        &self.0
    }

    /// Signal emitted when a particular action is invoked.
    ///
    /// # Specifications
    ///
    /// See also [`ActionInvoked`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-signal-org-freedesktop-portal-Notification.ActionInvoked).
    #[doc(alias = "ActionInvoked")]
    #[doc(alias = "XdpPortal::notification-action-invoked")]
    pub async fn receive_action_invoked(&self) -> Result<Action, Error> {
        receive_signal(self.inner(), "ActionInvoked").await
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
    ///
    /// # Specifications
    ///
    /// See also [`AddNotification`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Notification.AddNotification).
    #[doc(alias = "AddNotification")]
    #[doc(alias = "xdp_portal_add_notification")]
    pub async fn add_notification(
        &self,
        id: &str,
        notification: Notification<'_>,
    ) -> Result<(), Error> {
        call_method(self.inner(), "AddNotification", &(id, notification)).await
    }

    /// Withdraws a notification.
    ///
    /// # Arguments
    ///
    /// * `id` - Application-provided ID for this notification.
    ///
    /// # Specifications
    ///
    /// See also [`RemoveNotification`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Notification.RemoveNotification).
    #[doc(alias = "RemoveNotification")]
    #[doc(alias = "xdp_portal_remove_notification")]
    pub async fn remove_notification(&self, id: &str) -> Result<(), Error> {
        call_method(self.inner(), "RemoveNotification", &(id)).await
    }
}

//! # Examples
//!
//! How to monitor if there's a new update and install it.
//! Only available for Flatpak applications.
//!
//! ```no_run
//! use ashpd::flatpak::update_monitor::{
//!     UpdateInfo, UpdateMonitorProxy, UpdateOptions, UpdateProgress,
//! };
//! use ashpd::flatpak::{CreateMonitorOptions, FlatpakProxy};
//! use zbus::{fdo::Result, Connection};
//! use ashpd::WindowIdentifier;
//!
//! fn main() -> Result<()> {
//!     let connection = Connection::new_session()?;
//!     let proxy = FlatpakProxy::new(&connection)?;
//!
//!     let monitor_handle = proxy.create_update_monitor(CreateMonitorOptions::default())?;
//!     let monitor = UpdateMonitorProxy::new(&connection, &monitor_handle)?;
//!
//!     monitor.on_progress(|p: UpdateProgress| {
//!         println!("{:#?}", p);
//!         if p.progress == Some(100) {
//!             monitor.close().unwrap();
//!         }
//!     })?;
//!
//!     monitor.on_update_available(|_: UpdateInfo| {
//!         monitor
//!             .update(WindowIdentifier::default(), UpdateOptions::default())
//!             .unwrap();
//!     })?;
//!
//!     Ok(())
//! }
//! ```
use crate::WindowIdentifier;
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{fdo::DBusProxy, fdo::Result, Connection};
use zvariant::ObjectPath;
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specficied options on an update request
///
/// Currently there are no possible options yet.
pub struct UpdateOptions {}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// A response containing the update information when an update is available.
pub struct UpdateInfo {
    #[zvariant(rename = "running-commit")]
    /// The currently running ostree commit.
    pub running_commit: String,
    #[zvariant(rename = "local-commit")]
    /// The locally installed ostree commit.
    pub local_commit: String,
    #[zvariant(rename = "remote-commit")]
    /// The available commit ot install.
    pub remote_commit: String,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Copy, Clone, Debug, Type)]
#[repr(u32)]
/// The update status.
pub enum UpdateStatus {
    /// Running.
    Running = 0,
    /// No update to install.
    Empty = 1,
    /// Done.
    Done = 2,
    /// Failed.
    Failed = 3,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// A response of the update progress signal.
pub struct UpdateProgress {
    /// The number of operations that the update consists of.
    pub n_ops: Option<u32>,
    /// The position of the currently active operation.
    pub op: Option<u32>,
    /// The progress of the currently active operation, as a number between 0 and 100.
    pub progress: Option<u32>,
    /// The overall status of the update.
    pub status: Option<UpdateStatus>,
    /// The error name, sent when status is `UpdateStatus::Failed`
    pub error: Option<String>,
    /// The error message, sent when status is `UpdateStatus::Failed`
    pub error_message: Option<String>,
}

/// The interface exposes some interactions with Flatpak on the host to the sandbox.
/// For example, it allows you to restart the applications or start a more sandboxed instance.
pub struct UpdateMonitorProxy<'a> {
    proxy: DBusProxy<'a>,
    connection: &'a Connection,
}

impl<'a> UpdateMonitorProxy<'a> {
    /// Creates a new request proxy.
    ///
    /// # Arguments
    ///
    /// * `connection` - A DBus session connection.
    /// * `handle` - An object path returned by a create_update_monitor call.
    pub fn new(connection: &'a Connection, handle: &'a ObjectPath) -> Result<Self> {
        let proxy = DBusProxy::new_for(connection, handle, "/org/freedesktop/portal/Flatpak")?;
        Ok(Self { proxy, connection })
    }

    /// A signal received when there's progress during the application update.
    // FIXME: refactor once zbus supports signals
    pub fn on_progress<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(UpdateProgress),
    {
        loop {
            let msg = self.connection.receive_message()?;
            let msg_header = msg.header()?;
            if msg_header.message_type()? == zbus::MessageType::Signal
                && msg_header.member()? == Some("Progress")
            {
                let response = msg.body::<UpdateProgress>()?;
                callback(response);
            }
        }
    }

    /// A signal received when there's an application update.
    // FIXME: refactor once zbus supports signals
    pub fn on_update_available<F>(&self, callback: F) -> Result<()>
    where
        F: FnOnce(UpdateInfo),
    {
        loop {
            let msg = self.connection.receive_message()?;
            let msg_header = msg.header()?;
            if msg_header.message_type()? == zbus::MessageType::Signal
                && msg_header.member()? == Some("UpdateAvailable")
            {
                let response = msg.body::<UpdateInfo>()?;
                callback(response);
                break;
            }
        }
        Ok(())
    }

    /// Ends the update monitoring and cancels any ongoing installation.
    pub fn close(&self) -> zbus::Result<()> {
        self.proxy.call("Close", &())?;
        Ok(())
    }

    /// Asks to install an update of the calling app.
    ///
    /// Note that updates are only allowed if the new version
    /// has the same permissions (or less) than the currently installed version
    pub fn update(
        &self,
        parent_window: WindowIdentifier,
        options: UpdateOptions,
    ) -> zbus::Result<()> {
        self.proxy.call("Update", &(parent_window, options))
    }
}

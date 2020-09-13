//! # Examples
//!
//! How to monitor if there's a new update and install it.
//! Only available for Flatpak applications.
//!
//! ```no_run
//! use libportal::flatpak::update_monitor::{
//!     UpdateInfo, UpdateMonitorProxy, UpdateOptions, UpdateProgress,
//! };
//! use libportal::flatpak::{CreateMonitorOptions, FlatpakProxy};
//! use libportal::zbus::{fdo::Result, Connection};
//! use libportal::WindowIdentifier;
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
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
pub struct UpdateOptions {}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
pub struct UpdateInfo {
    #[zvariant(rename = "running-commit")]
    pub running_commit: Option<String>,
    #[zvariant(rename = "local-commit")]
    pub local_commit: Option<String>,
    #[zvariant(rename = "remote-commit")]
    pub remote_commit: Option<String>,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Copy, Clone, Debug, Type)]
#[repr(u32)]
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
    pub fn new(connection: &'a Connection, handle: &'a str) -> Result<Self> {
        let proxy = DBusProxy::new_for(connection, handle, "/org/freedesktop/portal/Flatpak")?;
        Ok(Self { proxy, connection })
    }

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

    ///  Ends the update monitoring and cancels any ongoing installation.
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

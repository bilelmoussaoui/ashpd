//! # Examples
//!
//! How to monitor if there's a new update and install it.
//! Only available for Flatpak applications.
//!
//! ```rust,no_run
//! use ashpd::flatpak::update_monitor::{
//!     UpdateInfo, UpdateMonitorProxy, UpdateOptions, UpdateProgress,
//! };
//! use ashpd::flatpak::{CreateMonitorOptions, FlatpakProxy};
//! use ashpd::WindowIdentifier;
//! use zbus::{fdo::Result, Connection};
//!
//! fn main() -> Result<()> {
//!     let connection = Connection::new_session()?;
//!     let proxy = FlatpakProxy::new(&connection)?;
//!
//!     let monitor = proxy.create_update_monitor(CreateMonitorOptions::default())?;
//!
//!     monitor.connect_progress(move |p: UpdateProgress| {
//!         if p.progress == Some(100) {
//!             monitor.close()?;
//!         }
//!         Ok(())
//!     })?;
//!
//!     monitor.connect_update_available(move |_: UpdateInfo| {
//!         monitor.update(WindowIdentifier::default(), UpdateOptions::default())?;
//!         Ok(())
//!     })?;
//!
//!     Ok(())
//! }
//! ```
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

use crate::WindowIdentifier;

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on an update request.
///
/// Currently there are no possible options yet.
pub struct UpdateOptions {}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// A response containing the update information when an update is available.
pub struct UpdateInfo {
    #[zvariant(rename = "running-commit")]
    /// The currently running OSTree commit.
    pub running_commit: String,
    #[zvariant(rename = "local-commit")]
    /// The locally installed OSTree commit.
    pub local_commit: String,
    #[zvariant(rename = "remote-commit")]
    /// The available commit to install.
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
    /// The progress of the currently active operation, as a number between 0
    /// and 100.
    pub progress: Option<u32>,
    /// The overall status of the update.
    pub status: Option<UpdateStatus>,
    /// The error name, sent when status is `UpdateStatus::Failed`.
    pub error: Option<String>,
    /// The error message, sent when status is `UpdateStatus::Failed`.
    pub error_message: Option<String>,
}

#[dbus_proxy(default_path = "/org/freedesktop/portal/Flatpak")]
/// The interface exposes some interactions with Flatpak on the host to the
/// sandbox. For example, it allows you to restart the applications or start a
/// more sandboxed instance.
trait UpdateMonitor {
    #[dbus_proxy(signal)]
    /// A signal received when there's progress during the application update.
    fn progress(&self, progress: UpdateProgress) -> Result<()>;

    #[dbus_proxy(signal)]
    /// A signal received when there's an application update.
    fn update_available(&self, update_info: UpdateInfo) -> Result<()>;

    /// Ends the update monitoring and cancels any ongoing installation.
    fn close(&self) -> Result<()>;

    /// Asks to install an update of the calling app.
    ///
    /// **Note** that updates are only allowed if the new version
    /// has the same permissions (or less) than the currently installed version.
    fn update(&self, parent_window: WindowIdentifier, options: UpdateOptions) -> Result<()>;
}

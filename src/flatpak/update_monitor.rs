//! # Examples
//!
//! How to monitor if there's a new update and install it.
//! Only available for Flatpak applications.
//!
//! ```rust,no_run
//! use ashpd::flatpak::FlatpakProxy;
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = FlatpakProxy::new(&connection).await?;
//!
//!     let monitor = proxy.create_update_monitor().await?;
//!     let info = monitor.receive_update_available().await?;
//!
//!     monitor.update(&WindowIdentifier::default()).await?;
//!     let progress = monitor.receive_progress().await?;
//!     println!("{:#?}", progress);
//!
//!     Ok(())
//! }
//! ```

use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::zvariant::{DeserializeDict, ObjectPath, SerializeDict, Type};

use super::DESTINATION;
use crate::{
    helpers::{call_method, receive_signal},
    Error, WindowIdentifier,
};

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`UpdateMonitorProxy::update`] request.
///
/// Currently there are no possible options yet.
#[zvariant(signature = "dict")]
struct UpdateOptions {}

#[derive(SerializeDict, DeserializeDict, Type, Debug)]
/// A response containing the update information when an update is available.
#[zvariant(signature = "dict")]
pub struct UpdateInfo {
    #[zvariant(rename = "running-commit")]
    /// The currently running OSTree commit.
    running_commit: String,
    #[zvariant(rename = "local-commit")]
    /// The locally installed OSTree commit.
    local_commit: String,
    #[zvariant(rename = "remote-commit")]
    /// The available commit to install.
    remote_commit: String,
}

impl UpdateInfo {
    pub fn running_commit(&self) -> &str {
        &self.running_commit
    }

    pub fn local_commit(&self) -> &str {
        &self.local_commit
    }

    pub fn remote_commit(&self) -> &str {
        &self.remote_commit
    }
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Copy, Clone, Debug, Type)]
#[repr(u32)]
/// The update status.
pub enum UpdateStatus {
    #[doc(alias = "XDP_UPDATE_STATUS_RUNNING")]
    /// Running.
    Running = 0,
    #[doc(alias = "XDP_UPDATE_STATUS_EMPTY")]
    /// No update to install.
    Empty = 1,
    #[doc(alias = "XDP_UPDATE_STATUS_DONE")]
    /// Done.
    Done = 2,
    #[doc(alias = "XDP_UPDATE_STATUS_FAILED")]
    /// Failed.
    Failed = 3,
}

#[derive(SerializeDict, DeserializeDict, Type, Debug)]
/// A response of the update progress signal.
#[zvariant(signature = "dict")]
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

/// The interface exposes some interactions with Flatpak on the host to the
/// sandbox. For example, it allows you to restart the applications or start a
/// more sandboxed instance.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Flatpak.UpdateMonitor`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Flatpak.UpdateMonitor).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Flatpak.UpdateMonitor")]
pub struct UpdateMonitorProxy<'a>(zbus::Proxy<'a>);

impl<'a> UpdateMonitorProxy<'a> {
    /// Create a new instance of [`UpdateMonitorProxy`].
    ///
    /// **Note** A [`UpdateMonitorProxy`] is not supposed to be created
    /// manually.
    pub(crate) async fn new(
        connection: &zbus::Connection,
        path: ObjectPath<'a>,
    ) -> Result<UpdateMonitorProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Flatpak.UpdateMonitor")?
            .path(path)?
            .destination(DESTINATION)?
            .build()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::Proxy<'_> {
        &self.0
    }

    /// A signal received when there's progress during the application update.
    ///
    /// # Specifications
    ///
    /// See also [`Progress`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-signal-org-freedesktop-portal-Flatpak-UpdateMonitor.Progress).
    #[doc(alias = "Progress")]
    #[doc(alias = "XdpPortal::update-progress")]
    pub async fn receive_progress(&self) -> Result<UpdateProgress, Error> {
        receive_signal(self.inner(), "Progress").await
    }

    /// A signal received when there's an application update.
    ///
    /// # Specifications
    ///
    /// See also [`UpdateAvailable`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-signal-org-freedesktop-portal-Flatpak-UpdateMonitor.UpdateAvailable).
    #[doc(alias = "UpdateAvailable")]
    #[doc(alias = "XdpPortal::update-available")]
    pub async fn receive_update_available(&self) -> Result<UpdateInfo, Error> {
        receive_signal(self.inner(), "UpdateAvailable").await
    }

    /// Asks to install an update of the calling app.
    ///
    /// **Note** updates are only allowed if the new version has the same
    /// permissions (or less) than the currently installed version.
    ///
    /// # Specifications
    ///
    /// See also [`Update`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Flatpak-UpdateMonitor.Update).
    #[doc(alias = "Update")]
    #[doc(alias = "xdp_portal_update_install")]
    pub async fn update(&self, identifier: &WindowIdentifier) -> Result<(), Error> {
        let options = UpdateOptions::default();
        call_method(self.inner(), "Update", &(&identifier, options)).await
    }

    /// Ends the update monitoring and cancels any ongoing installation.
    ///
    /// # Specifications
    ///
    /// See also [`Close`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Flatpak-UpdateMonitor.Close).
    #[doc(alias = "Close")]
    pub async fn close(&self) -> Result<(), Error> {
        call_method(self.inner(), "Close", &()).await
    }
}

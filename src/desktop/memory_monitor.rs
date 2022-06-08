//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::memory_monitor::MemoryMonitor;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let proxy = MemoryMonitor::new().await?;
//!
//!     let level = proxy.receive_low_memory_warning().await?;
//!     println!("{}", level);
//!
//!     Ok(())
//! }
//! ```

use super::{DESTINATION, PATH};
use crate::{
    helpers::{receive_signal, session_connection},
    Error,
};

/// The interface provides information about low system memory to sandboxed
/// applications. It is not a portal in the strict sense, since it does not
/// involve user interaction.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.MemoryMonitor`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.MemoryMonitor).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.MemoryMonitor")]
pub struct MemoryMonitor<'a>(zbus::Proxy<'a>);

impl<'a> MemoryMonitor<'a> {
    /// Create a new instance of [`MemoryMonitor`].
    pub async fn new() -> Result<MemoryMonitor<'a>, Error> {
        let connection = session_connection().await?;
        let proxy = zbus::ProxyBuilder::new_bare(&connection)
            .interface("org.freedesktop.portal.MemoryMonitor")?
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

    /// Signal emitted when a particular low memory situation happens
    /// with 0 being the lowest level of memory availability warning, and 255
    /// being the highest.
    ///
    /// # Specifications
    ///
    /// See also [`LowMemoryWarning`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-signal-org-freedesktop-portal-MemoryMonitor.LowMemoryWarning).
    #[doc(alias = "LowMemoryWarning")]
    pub async fn receive_low_memory_warning(&self) -> Result<i32, Error> {
        receive_signal(self.inner(), "LowMemoryWarning").await
    }
}

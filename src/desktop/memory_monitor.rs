//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::memory_monitor::MemoryMonitorProxy;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = MemoryMonitorProxy::new(&connection).await?;
//!
//!     let level = proxy.receive_low_memory_warning().await?;
//!     println!("{:#?}", level);
//!
//!     Ok(())
//! }
//! ```

use super::{DESTINATION, PATH};
use crate::{helpers::receive_signal, Error};

/// The interface provides information about low system memory to sandboxed
/// applications. It is not a portal in the strict sense, since it does not
/// involve user interaction.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.MemoryMonitor`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.MemoryMonitor).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.MemoryMonitor")]
pub struct MemoryMonitorProxy<'a>(zbus::Proxy<'a>);

impl<'a> MemoryMonitorProxy<'a> {
    /// Create a new instance of [`MemoryMonitorProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<MemoryMonitorProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
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

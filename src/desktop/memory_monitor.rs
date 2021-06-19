//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::memory_monitor::MemoryMonitorProxy;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = MemoryMonitorProxy::new(&connection).await?;
//!
//!     let level = proxy.receive_low_memory_warning().await?;
//!     println!("{:#?}", level);
//!
//!     Ok(())
//! }
//! ```

use crate::{helpers::property, Error};
use futures_lite::StreamExt;

/// The interface provides information about low system memory to sandboxed
/// applications. It is not a portal in the strict sense, since it does not
/// involve user interaction.
pub struct MemoryMonitorProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> MemoryMonitorProxy<'a> {
    pub async fn new(
        connection: &zbus::azync::Connection,
    ) -> Result<MemoryMonitorProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.MemoryMonitor")
            .path("/org/freedesktop/portal/desktop")?
            .destination("org.freedesktop.portal.Desktop")
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Signal emitted when a particular low memory situation happens
    /// with 0 being the lowest level of memory availability warning, and 255
    /// being the highest.
    pub async fn receive_low_memory_warning(&self) -> Result<i32, Error> {
        let mut stream = self.0.receive_signal("LowMemoryWarning").await?;
        let message = stream.next().await.ok_or(Error::NoResponse)?;
        message.body::<i32>().map_err(From::from)
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}

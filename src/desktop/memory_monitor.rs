//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::memory_monitor::MemoryMonitorProxy;
//! use zbus::{self, fdo::Result};
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = MemoryMonitorProxy::new(&connection)?;
//!     proxy.connect_low_memory_warning(move |level| {
//!         println!("{:#?}", level);
//!         Ok(())
//!     })?;
//!     Ok(())
//! }
//! ```
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.MemoryMonitor",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface provides information about low system memory to sandboxed
/// applications. It is not a portal in the strict sense, since it does not
/// involve user interaction.
trait MemoryMonitor {
    #[dbus_proxy(signal)]
    /// Signal emitted when a particular low memory situation happens
    /// with 0 being the lowest level of memory availability warning, and 255
    /// being the highest.
    fn low_memory_warning(&self, level: i32) -> Result<()>;

    /// The version of this DBus interface.
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

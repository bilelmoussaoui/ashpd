//! # Examples
//!
//! ```no_run
//! use libportal::desktop::memory_monitor::MemoryMonitorProxy;
//! use libportal::zbus::{self, fdo::Result};
//!
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = MemoryMonitorProxy::new(&connection)?;
//!     proxy.on_low_memory_warning(move |level| {
//!         println!("{:#?}", level);
//!     })?;
//!     Ok(())
//! }
//! ```

use zbus::{fdo::Result, Connection, Proxy};
/// The interface provides information about low system memory to sandboxed applications.
/// It is not a portal in the strict sense, since it does not involve user interaction.
pub struct MemoryMonitorProxy<'a> {
    proxy: Proxy<'a>,
    connection: &'a Connection,
}

impl<'a> MemoryMonitorProxy<'a> {
    pub fn new(connection: &'a Connection) -> Result<Self> {
        let proxy = Proxy::new(
            connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.MemoryMonitor",
        )?;
        Ok(Self { proxy, connection })
    }

    /// Signal emitted when a particular low memory situation happens with 0 being the lowest level of memory availability warning, and 255 being the highest
    // FIXME: refactor once zbus supports signals
    pub fn on_low_memory_warning<F>(&self, callback: F) -> Result<()>
    where
        F: FnOnce(u32),
    {
        loop {
            let msg = self.connection.receive_message()?;
            let msg_header = msg.header()?;
            if msg_header.message_type()? == zbus::MessageType::Signal
                && msg_header.member()? == Some("LowMemoryWarning")
            {
                let response = msg.body::<u32>()?;
                callback(response);
                break;
            }
        }
        Ok(())
    }

    /// version property
    pub fn version(&self) -> Result<u32> {
        self.proxy.get_property::<u32>("version")
    }
}

use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.MemoryMonitor",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface provides information about low system memory to sandboxed applications.
/// It is not a portal in the strict sense, since it does not involve user interaction.
/// Applications are expected to use this interface indirectly, via a library API such as the GLib GMemoryMonitor interface.
trait MemoryMonitor {
    // FIXME: enable once signals are supported
    // Signal emitted when a particular low memory situation happens with 0 being the lowest level of memory availability warning, and 255 being the highest
    // fn low_memory_warning(&self, level: u32);

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

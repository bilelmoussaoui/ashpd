use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(interface = "org.freedesktop.portal.Session")]
trait Session {
    fn close(&self) -> Result<()>;
    // signal
    //fn closed(&self) -> Result<HashMap<&str, Value>>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

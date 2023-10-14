use zbus::{dbus_interface, SignalContext};

pub struct Session {}

#[dbus_interface(name = "org.freedesktop.impl.portal.Session")]
impl Session {
    async fn close(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }

    #[dbus_interface(property)]
    fn version(&self) -> u32 {
        2
    }

    #[dbus_interface(signal)]
    async fn closed(signal_ctxt: &SignalContext<'_>, message: &str) -> zbus::Result<()>;
}

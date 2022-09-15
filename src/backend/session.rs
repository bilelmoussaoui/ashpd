use zbus::SignalContext;

pub struct Session {}

#[zbus::interface(name = "org.freedesktop.impl.portal.Session")]
impl Session {
    async fn close(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }

    #[zbus(property)]
    fn version(&self) -> u32 {
        2
    }

    #[zbus(signal)]
    async fn closed(signal_ctxt: &SignalContext<'_>, message: &str) -> zbus::Result<()>;
}

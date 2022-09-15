use zbus::dbus_interface;

pub struct Request {}

#[dbus_interface(name = "org.freedesktop.impl.portal.Request")]
impl Request {
    async fn close(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }
}

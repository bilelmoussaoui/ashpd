use zbus::{dbus_proxy, fdo::Result};

pub enum ResponseType {
    // Success, the request is carried out
    Success = 0,
    // The user cancelled the interaction
    Cancelled = 1,
    // The user interaction was ended in some other way
    Other = 2,
}

#[dbus_proxy(interface = "org.freedesktop.portal.Request")]
trait Request {
    fn close(&self) -> Result<()>;
    // signal
    //fn response(&self) -> Result<(ResponseType, HashMap<&str, Value>)>;
}

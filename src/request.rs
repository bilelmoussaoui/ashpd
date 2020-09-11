use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{fdo::DBusProxy, fdo::Result, Connection};
use zvariant_derive::Type;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
pub enum ResponseType {
    /// Success, the request is carried out
    Success = 0,
    /// The user cancelled the interaction
    Cancelled = 1,
    /// The user interaction was ended in some other way
    Other = 2,
}

/// The Request interface is shared by all portal interfaces.
/// When a portal method is called, the reply includes a handle (i.e. object path) for a Request object,
/// which will stay alive for the duration of the user interaction related to the method call.
///
/// The portal indicates that a portal request interaction is over by emitting the "Response" signal on the Request object.
///
/// The application can abort the interaction calling Close() on the Request object.
///
/// Since version 0.9 of xdg-desktop-portal, the handle will be of the form
/// `/org/freedesktop/portal/desktop/request/SENDER/TOKEN`, where SENDER is the callers unique name,
///  with the initial ':' removed and all '.' replaced by '_',
/// and TOKEN is a unique token that the caller provided with the handle_token key in the options vardict.
///
/// This change was made to let applications subscribe to the Response signal before
/// making the initial portal call, thereby avoiding a race condition.
/// It is recommended that the caller should verify that the returned handle is what
/// it expected, and update its signal subscription if it isn't.
/// This ensures that applications will work with both old and new versions of xdg-desktop-portal.
pub struct RequestProxy<'a> {
    proxy: DBusProxy<'a>,
    connection: &'a Connection,
}

impl<'a> RequestProxy<'a> {
    pub fn new(connection: &'a Connection, handle: &'a str) -> Result<Self> {
        let proxy = DBusProxy::new_for(connection, handle, "/org/freedesktop/portal/desktop")?;
        Ok(Self { proxy, connection })
    }

    pub fn on_response<F, T>(&self, callback: F) -> Result<()>
    where
        F: FnOnce(T),
        T: serde::de::DeserializeOwned + zvariant::Type,
    {
        loop {
            let msg = self.connection.receive_message()?;
            let msg_header = msg.header()?;
            if msg_header.message_type()? == zbus::MessageType::Signal
                && msg_header.member()? == Some("Response")
            {
                let response = msg.body::<T>()?;
                callback(response);
                break;
            }
        }
        Ok(())
    }

    /// Closes the portal request to which this object refers and ends all related user interaction (dialogs, etc).
    /// A Response signal will not be emitted in this case.
    pub fn close(&self) -> Result<()> {
        self.proxy.call("Close", &())?;
        Ok(())
    }
}

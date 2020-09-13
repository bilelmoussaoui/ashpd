use std::collections::HashMap;
use zbus::{fdo::DBusProxy, fdo::Result, Connection};
use zvariant::OwnedValue;

pub type SessionDetails = HashMap<String, OwnedValue>;

/// The Session interface is shared by all portal interfaces that involve long lived sessions.
/// When a method that creates a session is called, if successful, the reply will include a session handle (i.e. object path) for a Session object, which will stay alive for the duration of the session.
///
/// The duration of the session is defined by the interface that creates it.
/// For convenience, the interface contains a method Close(), and a signal `org.freedesktop.portal.Session::Closed`.
/// Whether it is allowed to directly call Close() depends on the interface.
///
/// The handle of a session will be of the form `/org/freedesktop/portal/desktop/session/SENDER/TOKEN`,
/// where SENDER is the callers unique name, with the initial ':' removed and all '.' replaced by '_',
/// and TOKEN is a unique token that the caller provided with the session_handle_token key in the options vardict of the method creating the session.
///
/// The token that the caller provides should be unique and not guessable.
/// To avoid clashes with calls made from unrelated libraries, it is a good idea to use a per-library prefix combined with a random number.
///
/// A client who started a session vanishing from the D-Bus is equivalent to closing all active sessions made by said client.
pub struct SessionProxy<'a> {
    proxy: DBusProxy<'a>,
    connection: &'a Connection,
}

impl<'a> SessionProxy<'a> {
    pub fn new(connection: &'a Connection, handle: &'a str) -> Result<Self> {
        let proxy = DBusProxy::new_for(connection, handle, "/org/freedesktop/portal/desktop")?;
        Ok(Self { proxy, connection })
    }

    /// Emitted when a session is closed.
    // FIXME: refactor once zbus supports signals
    pub fn on_closed<F>(&self, callback: F) -> Result<()>
    where
        F: FnOnce(SessionDetails) -> Result<()>,
    {
        loop {
            let msg = self.connection.receive_message()?;
            let msg_header = msg.header()?;
            if msg_header.message_type()? == zbus::MessageType::Signal
                && msg_header.member()? == Some("Closed")
            {
                let response = msg.body::<SessionDetails>()?;
                callback(response)?;
                break;
            }
        }
        Ok(())
    }

    /// Closes the portal session to which this object refers and ends all related user interaction (dialogs, etc).
    pub fn close(&self) -> Result<()> {
        self.proxy.call("Close", &())?;
        Ok(())
    }

    /// version property
    pub fn version(&self) -> Result<u32> {
        self.proxy.get_property::<u32>("version")
    }
}

use crate::{HandleToken, WindowIdentifier};
use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{fdo::Result, Connection, Proxy};
use zvariant::{ObjectPath, OwnedObjectPath};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a create inhibit monitor request.
pub struct CreateMonitorOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<HandleToken>,
    /// A string that will be used as the last element of the session handle.
    pub session_handle_token: Option<HandleToken>,
}

impl CreateMonitorOptions {
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    pub fn session_handle_token(mut self, session_handle_token: HandleToken) -> Self {
        self.session_handle_token = Some(session_handle_token);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options of an inhibit request.
pub struct InhibitOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<HandleToken>,
    /// User-visible reason for the inhibition.
    pub reason: Option<String>,
}

impl InhibitOptions {
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    pub fn reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Copy, BitFlags, Type)]
#[repr(u32)]
pub enum InhibitFlags {
    Logout = 1,
    UserSwitch = 2,
    Suspend = 4,
    Idle = 8,
}

#[derive(Debug, SerializeDict, DeserializeDict, TypeDict)]
struct InhibitMonitorResponse {
    pub session_handle: OwnedObjectPath,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct InhibitState(OwnedObjectPath, State);

impl InhibitState {
    pub fn session_handle(&self) -> OwnedObjectPath {
        self.0.clone()
    }

    pub fn screensaver_active(&self) -> bool {
        self.1.screensaver_active
    }

    pub fn session_state(&self) -> SessionState {
        self.1.session_state
    }
}

#[derive(Debug, SerializeDict, DeserializeDict, TypeDict)]
struct State {
    #[zvariant(rename = "screensaver-active")]
    pub screensaver_active: bool,
    #[zvariant(rename = "session-state")]
    pub session_state: SessionState,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Copy, Type)]
#[repr(u32)]
pub enum SessionState {
    Running = 1,
    QueryEnd = 2,
    Ending = 3,
}

/// The interface lets sandboxed applications inhibit the user session from ending, suspending, idling or getting switched away.
pub struct InhibitProxy<'a> {
    proxy: Proxy<'a>,
    connection: &'a Connection,
}

impl<'a> InhibitProxy<'a> {
    pub fn new(connection: &'a Connection) -> Result<Self> {
        let proxy = Proxy::new(
            connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Inhibit",
        )?;
        Ok(Self { proxy, connection })
    }

    // Signal emitted when a particular low memory situation happens with 0 being the lowest level of memory availability warning, and 255 being the highest
    pub fn on_state_changed<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(&InhibitProxy, InhibitState) -> Result<()>,
    {
        loop {
            let msg = self.connection.receive_message()?;
            let msg_header = msg.header()?;
            if msg_header.message_type()? == zbus::MessageType::Signal
                && msg_header.member()? == Some("StateChanged")
            {
                let response = msg.body::<InhibitState>()?;
                callback(self, response)?;
            }
        }
    }

    /// Creates a monitoring session.
    /// While this session is active, the caller will receive `state_changed` signals
    /// with updates on the session state.
    ///
    /// Returns a [`RequestProxy`] object path.
    ///
    /// # Arguments
    ///
    /// * `window` - The application window identifier
    /// * `options` - [`CreateMonitorOptions`]
    ///
    /// [`CreateMonitorOptions`]: ./struct.CreateMonitorOptions.html
    /// [`RequestProxy`]: ../request/struct.RequestProxy.html
    pub fn create_monitor(
        &self,
        window: WindowIdentifier,
        options: CreateMonitorOptions,
    ) -> zbus::Result<OwnedObjectPath> {
        self.proxy.call("CreateMonitor", &(window, options))
    }

    /// Inhibits a session status changes.
    ///
    /// Returns a [`RequestProxy`] object path.
    ///
    /// # Arguments
    ///
    /// * `window` - The application window identifier
    /// * `flags` - The flags determine what changes are inhibited
    /// * `options` - [`InhibitOptions`]
    ///
    /// [`InhibitOptions`]: ./struct.InhibitOptions.html
    /// [`RequestProxy`]: ../request/struct.RequestProxy.html
    pub fn inhibit(
        &self,
        window: WindowIdentifier,
        flags: BitFlags<InhibitFlags>,
        options: InhibitOptions,
    ) -> zbus::Result<OwnedObjectPath> {
        self.proxy.call("Inhibit", &(window, flags, options))
    }

    /// QueryEndResponse method
    pub fn query_end_response(&self, session_handle: ObjectPath) -> zbus::Result<()> {
        self.proxy.call("QueryEndResponse", &(session_handle))
    }

    /// version property
    pub fn version(&self) -> Result<u32> {
        self.proxy.get_property::<u32>("version")
    }
}

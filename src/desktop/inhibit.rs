use crate::{ResponseType, WindowIdentifier};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{ObjectPath, OwnedObjectPath};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a create inhibit monitor request.
pub struct CreateMonitorOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
    /// A string that will be used as the last element of the session handle.
    pub session_handle_token: Option<String>,
}

impl CreateMonitorOptions {
    pub fn handle_token(mut self, handle_token: &str) -> Self {
        self.handle_token = Some(handle_token.to_string());
        self
    }

    pub fn session_handle_token(mut self, session_handle_token: &str) -> Self {
        self.session_handle_token = Some(session_handle_token.to_string());
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options of an inhibit request.
pub struct InhibitOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
    /// User-visible reason for the inhibition.
    pub reason: Option<String>,
}

impl InhibitOptions {
    pub fn handle_token(mut self, handle_token: &str) -> Self {
        self.handle_token = Some(handle_token.to_string());
        self
    }

    pub fn reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
pub enum InhibitFlags {
    Logout = 1,
    UserSwitch = 2,
    Suspend = 3,
    Idle = 4,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct InhibitMonitorResponse(ResponseType, InhibitMonitorResult);

impl InhibitMonitorResponse {
    pub fn session_handle<'a>(&self) -> &'a ObjectPath {
        &self.1.session_handle
    }
}

#[derive(Debug, SerializeDict, DeserializeDict, TypeDict)]
struct InhibitMonitorResult {
    pub session_handle: OwnedObjectPath,
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Inhibit",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications inhibit the user session from ending, suspending, idling or getting switched away.
trait Inhibit {
    /// Creates a monitoring session.
    /// While this session is active, the caller will receive `state_changed` signals
    /// with updates on the session state.
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `window` - The application window identifier
    /// * `options` - [`CreateMonitorOptions`]
    ///
    /// [`CreateMonitorOptions`]: ./struct.CreateMonitorOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn create_monitor(
        &self,
        window: WindowIdentifier,
        options: CreateMonitorOptions,
    ) -> Result<OwnedObjectPath>;

    /// Inhibits a session status changes.
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `window` - The application window identifier
    /// * `flags` - The flags determine what changes are inhibited
    /// * `options` - [`InhibitOptions`]
    ///
    /// [`InhibitOptions`]: ./struct.InhibitOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn inhibit(
        &self,
        window: WindowIdentifier,
        flags: InhibitFlags,
        options: InhibitOptions,
    ) -> Result<OwnedObjectPath>;

    /// QueryEndResponse method
    fn query_end_response(&self, session_handle: ObjectPath) -> Result<()>;

    // signal
    // fn state_changed(&self, session_handle: ObjectPath, )

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

use crate::WindowIdentifier;
use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options for a create inhibit monitor request.
pub struct CreateMonitorOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
    /// A string that will be used as the last element of the session handle.
    pub session_handle_token: Option<String>,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options of an inhibit request.
pub struct InhibitOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
    /// User-visible reason for the inhibition.
    pub reason: String,
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
    /// * `parent_window` - The application window identifier
    /// * `options` - [`CreateMonitorOptions`]
    ///
    /// [`CreateMonitorOptions`]: ./struct.CreateMonitorOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn create_monitor(
        &self,
        parent_window: WindowIdentifier,
        options: CreateMonitorOptions,
    ) -> Result<String>;

    /// Inhibits a session status changes.
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - The application window identifier
    /// * `flags` - The flags determine what changes are inhibited
    ///     1 - Logout
    ///     2 - User switch
    ///     3 - Suspend
    ///     4 - Idle
    ///     FIXME: switch to an enum
    /// * `options` - [`InhibitOptions`]
    ///
    /// [`InhibitOptions`]: ./struct.InhibitOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn inhibit(
        &self,
        parent_window: WindowIdentifier,
        flags: u32,
        options: InhibitOptions,
    ) -> Result<String>;

    /// QueryEndResponse method
    fn query_end_response(&self, session_handle: &str) -> Result<()>;

    // signal
    // fn state_changed(&self, session_handle: &str, )

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

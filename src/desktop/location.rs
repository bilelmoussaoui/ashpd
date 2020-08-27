use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
#[zvariant(deny_unknown_fields)]
pub struct LocationAccessOptions {
    /// A string that will be used as the last element of the session handle.
    pub session_handle_token: Option<String>,
    /// Distance threshold in meters. Default is 0.
    pub distance_threshold: Option<u32>,
    /// Time threshold in seconds. Default is 0.
    pub time_threshold: Option<u32>,
    /// Requested accuracy. Default is EXACT.
    /// Values: NONE 0, COUNTRY 1, CITY 2, NEIGHBORHOOD 3, STREET 4, EXACT 5
    /// FIXME: switch to an enum
    pub accuracy: Option<u32>,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
#[zvariant(deny_unknown_fields)]
pub struct LocationStartOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Location",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications query basic information about the location.
trait Location {
    /// Create a location session.
    ///
    /// Returns a [`Session`] handle
    ///
    /// # Arguments
    ///
    /// * `options` - A [`LocationAccessOptions`]
    ///
    /// [`LocationAccessOptions`]: ./struct.LocationAccessOptions.html
    /// [`Session`]: ../session/struct.SessionProxy.html
    fn create_session(&self, options: LocationAccessOptions) -> Result<String>;

    /// Start the location session.
    /// An application can only attempt start a session once.
    ///
    /// Returns a [`Session`] handle
    ///
    /// # Arguments
    ///
    /// * `session_handle` - Object path of the [`Session`] object
    /// * `parent_window` - Identifier for the application window
    /// * `options` - A `LocationStartOptions`
    ///
    /// [`Session`]: ../session/struct.SessionProxy.html
    fn start(
        &self,
        session_handle: &str,
        parent_window: &str,
        options: LocationStartOptions,
    ) -> Result<String>;

    // signal
    // fn location_updated(&self, session_handle: &str, location: HashMap<&str, Value>);

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

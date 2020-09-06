use crate::WindowIdentifier;
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{ObjectPath, OwnedObjectPath};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
pub enum Accuracy {
    None = 0,
    Country = 1,
    City = 2,
    Neighborhood = 3,
    Street = 4,
    Exact = 5,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a location access request.
pub struct LocationAccessOptions {
    /// A string that will be used as the last element of the session handle.
    pub session_handle_token: Option<String>,
    /// Distance threshold in meters. Default is 0.
    pub distance_threshold: Option<u32>,
    /// Time threshold in seconds. Default is 0.
    pub time_threshold: Option<u32>,
    /// Requested accuracy. Default is `Accuracy::Exact`.
    pub accuracy: Option<Accuracy>,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a location session start request.
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
    fn create_session(&self, options: LocationAccessOptions) -> Result<OwnedObjectPath>;

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
        session_handle: ObjectPath,
        parent_window: WindowIdentifier,
        options: LocationStartOptions,
    ) -> Result<OwnedObjectPath>;

    // signal
    // fn location_updated(&self, session_handle: ObjectPath, location: HashMap<&str, Value>);

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

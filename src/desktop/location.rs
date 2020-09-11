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

impl LocationAccessOptions {
    pub fn session_handle_token(mut self, session_handle_token: &str) -> Self {
        self.session_handle_token = Some(session_handle_token.to_string());
        self
    }

    pub fn distance_threshold(mut self, distance_threshold: u32) -> Self {
        self.distance_threshold = Some(distance_threshold);
        self
    }

    pub fn time_threshold(mut self, time_threshold: u32) -> Self {
        self.time_threshold = Some(time_threshold);
        self
    }

    pub fn accuracy(mut self, accuracy: Accuracy) -> Self {
        self.accuracy = Some(accuracy);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a location session start request.
pub struct LocationStartOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
}

impl LocationStartOptions {
    pub fn handle_token(mut self, handle_token: &str) -> Self {
        self.handle_token = Some(handle_token.to_string());
        self
    }
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

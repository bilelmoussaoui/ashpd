//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::location::{LocationAccessOptions, LocationProxy, LocationStartOptions};
//! use ashpd::{BasicResponse as Basic, HandleToken, RequestProxy, Response, WindowIdentifier};
//! use std::convert::TryFrom;
//! use zbus::{self, fdo::Result};
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = LocationProxy::new(&connection)?;
//!
//!     let options = LocationAccessOptions::default()
//!         .session_handle_token(HandleToken::try_from("token").unwrap());
//!
//!     let session = proxy.create_session(options)?;
//!
//!     let request = proxy.start(
//!         session.path(),
//!         WindowIdentifier::default(),
//!         LocationStartOptions::default(),
//!     )?;
//!
//!     request.connect_response(move |response: Response<Basic>| {
//!         proxy.connect_location_updated(move |location| {
//!             println!("{}", location.accuracy());
//!             println!("{}", location.longitude());
//!             println!("{}", location.latitude());
//!             Ok(())
//!         })?;
//!         Ok(())
//!     })?;
//!
//!     Ok(())
//! }
//! ```
use crate::{
    AsyncRequestProxy, AsyncSessionProxy, HandleToken, RequestProxy, SessionProxy, WindowIdentifier,
};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{ObjectPath, OwnedObjectPath};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
/// The accuracy of the location.
pub enum Accuracy {
    /// None.
    None = 0,
    /// Country.
    Country = 1,
    /// City.
    City = 2,
    /// Neighborhood.
    Neighborhood = 3,
    /// Street.
    Street = 4,
    /// The exact location.
    Exact = 5,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a location access request.
pub struct LocationAccessOptions {
    /// A string that will be used as the last element of the session handle.
    session_handle_token: Option<HandleToken>,
    /// Distance threshold in meters. Default is 0.
    distance_threshold: Option<u32>,
    /// Time threshold in seconds. Default is 0.
    time_threshold: Option<u32>,
    /// Requested accuracy. Default is `Accuracy::Exact`.
    accuracy: Option<Accuracy>,
}

impl LocationAccessOptions {
    /// Sets the session handle token.
    pub fn session_handle_token(mut self, session_handle_token: HandleToken) -> Self {
        self.session_handle_token = Some(session_handle_token);
        self
    }

    /// Sets the distance threshold in meters.
    pub fn distance_threshold(mut self, distance_threshold: u32) -> Self {
        self.distance_threshold = Some(distance_threshold);
        self
    }

    /// Sets the time threshold in seconds.
    pub fn time_threshold(mut self, time_threshold: u32) -> Self {
        self.time_threshold = Some(time_threshold);
        self
    }

    /// Sets the location accuracy.
    pub fn accuracy(mut self, accuracy: Accuracy) -> Self {
        self.accuracy = Some(accuracy);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a location session start request.
pub struct LocationStartOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
}

impl LocationStartOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Type)]
/// The response received on a `location_updated` signal.
pub struct LocationResponse(OwnedObjectPath, Location);

impl LocationResponse {
    /// A `SessionProxy` object path.
    pub fn session_handle<'a>(&self) -> &'a ObjectPath {
        &self.0
    }

    /// The accuracy, in meters.
    pub fn accuracy(&self) -> f64 {
        self.1.accuracy
    }

    /// The altitude, in meters.
    pub fn altitude(&self) -> f64 {
        self.1.altitude
    }

    /// The speed, in meters per second.
    pub fn speed(&self) -> f64 {
        self.1.speed
    }

    /// The heading, in degrees, going clockwise. North 0, East 90, South 180, West 270.
    pub fn heading(&self) -> f64 {
        self.1.heading
    }

    /// The location description
    pub fn description(&self) -> &str {
        &self.1.description
    }

    /// The latitude, in degrees.
    pub fn latitude(&self) -> f64 {
        self.1.latitude
    }

    /// The longitude, in degrees.
    pub fn longitude(&self) -> f64 {
        self.1.longitude
    }

    /// The timestamp, as seconds.
    pub fn timestamp(&self) -> u64 {
        self.1.timestamp.0
    }
}

#[derive(Debug, SerializeDict, DeserializeDict, TypeDict)]
struct Location {
    #[zvariant(rename = "Accuracy")]
    accuracy: f64,
    #[zvariant(rename = "Altitude")]
    altitude: f64,
    #[zvariant(rename = "Speed")]
    speed: f64,
    #[zvariant(rename = "Heading")]
    heading: f64,
    #[zvariant(rename = "Description")]
    description: String,
    #[zvariant(rename = "Latitude")]
    latitude: f64,
    #[zvariant(rename = "Longitude")]
    longitude: f64,
    #[zvariant(rename = "Timestamp")]
    timestamp: (u64, u64),
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Location",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications query basic information about the location.
trait Location {
    #[dbus_proxy(signal)]
    /// Signal emitted when the user location is updated.
    fn location_updated(&self, location: LocationResponse) -> Result<()>;

    /// Create a location session.
    ///
    /// # Arguments
    ///
    /// * `options` - A [`LocationAccessOptions`]
    ///
    /// [`LocationAccessOptions`]: ./struct.LocationAccessOptions.html
    #[dbus_proxy(object = "Session")]
    fn create_session(&self, options: LocationAccessOptions);

    /// Start the location session.
    /// An application can only attempt start a session once.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `parent_window` - Identifier for the application window.
    /// * `options` - A `LocationStartOptions`.
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    #[dbus_proxy(object = "Request")]
    fn start(
        &self,
        session_handle: &ObjectPath<'_>,
        parent_window: WindowIdentifier,
        options: LocationStartOptions,
    );

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

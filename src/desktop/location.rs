//! # Examples
//!
//! ```no_run
//! use libportal::desktop::location::{
//!     LocationAccessOptions, LocationProxy, LocationResponse, LocationStartOptions,
//! };
//! use libportal::{
//!     zbus::{self, fdo::Result},
//!     RequestProxy, Response, WindowIdentifier,
//! };
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = LocationProxy::new(&connection)?;
//!
//!     let options = LocationAccessOptions::default()
//!                     .session_handle_token("token");
//!
//!     let session_handle = proxy.create_session(options)?;
//!
//!     let request_handle = proxy.start(
//!         session_handle.into(),
//!         WindowIdentifier::default(),
//!         LocationStartOptions::default(),
//!     )?;
//!
//!     let request = RequestProxy::new(&connection, &request_handle)?;
//!     request.on_response(move |response: Response| -> Result<()> {
//!         if response.is_success() {
//!             proxy.on_location_updated(move |location: LocationResponse| -> Result<()> {
//!                 println!("{}", location.accuracy());
//!                 println!("{}", location.longitude());
//!                 println!("{}", location.latitude());
//!                 Ok(())
//!             })?;
//!         }
//!         Ok(())
//!     })?;
//!
//!     Ok(())
//! }
//! ```
use crate::WindowIdentifier;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{fdo::Result, Connection, Proxy};
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

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct LocationResponse(OwnedObjectPath, Location);

impl LocationResponse {
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

    pub fn description(&self) -> String {
        self.1.description.clone()
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

pub struct LocationProxy<'a> {
    proxy: Proxy<'a>,
    connection: &'a Connection,
}

impl<'a> LocationProxy<'a> {
    pub fn new(connection: &'a Connection) -> Result<Self> {
        let proxy = Proxy::new(
            connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Location",
        )?;
        Ok(Self { proxy, connection })
    }

    pub fn on_location_updated<F, T>(&self, callback: F) -> Result<()>
    where
        F: FnOnce(T) -> Result<()>,
        T: serde::de::DeserializeOwned + zvariant::Type,
    {
        loop {
            let msg = self.connection.receive_message()?;
            let msg_header = msg.header()?;
            if msg_header.message_type()? == zbus::MessageType::Signal
                && msg_header.member()? == Some("LocationUpdated")
            {
                let response = msg.body::<T>()?;
                callback(response)?;
                break;
            }
        }
        Ok(())
    }

    /// Create a location session.
    ///
    /// Returns a [`SessionProxy`] object path.
    ///
    /// # Arguments
    ///
    /// * `options` - A [`LocationAccessOptions`]
    ///
    /// [`LocationAccessOptions`]: ./struct.LocationAccessOptions.html
    /// [`SessionProxy`]: ../session/struct.SessionProxy.html
    pub fn create_session(&self, options: LocationAccessOptions) -> zbus::Result<OwnedObjectPath> {
        self.proxy.call("CreateSession", &(options))
    }

    /// Start the location session.
    /// An application can only attempt start a session once.
    ///
    /// Returns a [`RequestProxy`] object path.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - Object path returned by `create_session`
    /// * `parent_window` - Identifier for the application window
    /// * `options` - A `LocationStartOptions`
    ///
    /// [`RequestProxy`]: ../session/struct.RequestProxy.html
    pub fn start(
        &self,
        session_handle: ObjectPath,
        parent_window: WindowIdentifier,
        options: LocationStartOptions,
    ) -> zbus::Result<OwnedObjectPath> {
        self.proxy
            .call("Start", &(session_handle, parent_window, options))
    }

    /// version property
    pub fn version(&self) -> Result<u32> {
        self.proxy.get_property::<u32>("version")
    }
}

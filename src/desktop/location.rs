//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::location::{Accuracy, LocationProxy};
//! use ashpd::WindowIdentifier;
//! use futures::TryFutureExt;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = LocationProxy::new(&connection).await?;
//!     let identifier = WindowIdentifier::default();
//!
//!     let session = proxy.create_session(None, None, Some(Accuracy::Street)).await?;
//!
//!     let (_, location) = futures::try_join!(
//!         proxy.start(&session, &identifier).into_future(),
//!         proxy.receive_location_updated().into_future()
//!     )?;
//!
//!     println!("{}", location.accuracy());
//!     println!("{}", location.longitude());
//!     println!("{}", location.latitude());
//!     session.close().await?;
//!
//!     Ok(())
//! }
//! ```

use std::fmt::Debug;

use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::zvariant::{DeserializeDict, OwnedObjectPath, SerializeDict, Type};

use super::{HandleToken, SessionProxy, DESTINATION, PATH};
use crate::{
    helpers::{call_basic_response_method, call_method, receive_signal},
    Error, WindowIdentifier,
};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Clone, Copy, Debug, Type)]
#[doc(alias = "XdpLocationAccuracy")]
#[repr(u32)]
/// The accuracy of the location.
pub enum Accuracy {
    #[doc(alias = "XDP_LOCATION_ACCURACY_NONE")]
    /// None.
    None = 0,
    #[doc(alias = "XDP_LOCATION_ACCURACY_COUNTRY")]
    /// Country.
    Country = 1,
    #[doc(alias = "XDP_LOCATION_ACCURACY_CITY")]
    /// City.
    City = 2,
    #[doc(alias = "XDP_LOCATION_ACCURACY_NEIGHBORHOOD")]
    /// Neighborhood.
    Neighborhood = 3,
    #[doc(alias = "XDP_LOCATION_ACCURACY_STREET")]
    /// Street.
    Street = 4,
    #[doc(alias = "XDP_LOCATION_ACCURACY_EXACT")]
    /// The exact location.
    Exact = 5,
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`LocationProxy::create_session`] request.
#[zvariant(signature = "dict")]
struct CreateSessionOptions {
    /// A string that will be used as the last element of the session handle.
    session_handle_token: HandleToken,
    /// Distance threshold in meters. Default is 0.
    #[zvariant(rename = "distance-threshold")]
    distance_threshold: Option<u32>,
    /// Time threshold in seconds. Default is 0.
    #[zvariant(rename = "time-threshold")]
    time_threshold: Option<u32>,
    /// Requested accuracy. Default is `Accuracy::Exact`.
    accuracy: Option<Accuracy>,
}

impl CreateSessionOptions {
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

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`LocationProxy::start`] request.
#[zvariant(signature = "dict")]
struct SessionStartOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
}

#[derive(Serialize, Deserialize, Type)]
/// The response received on a `location_updated` signal.
pub struct Location(OwnedObjectPath, LocationInner);

impl Location {
    /// The accuracy, in meters.
    pub fn accuracy(&self) -> f64 {
        self.1.accuracy
    }

    /// The altitude, in meters.
    pub fn altitude(&self) -> Option<f64> {
        if self.1.altitude == -f64::MAX {
            None
        } else {
            Some(self.1.altitude)
        }
    }

    /// The speed, in meters per second.
    pub fn speed(&self) -> Option<f64> {
        if self.1.speed == -1f64 {
            None
        } else {
            Some(self.1.speed)
        }
    }

    /// The heading, in degrees, going clockwise. North 0, East 90, South 180,
    /// West 270.
    pub fn heading(&self) -> Option<f64> {
        if self.1.heading == -1f64 {
            None
        } else {
            Some(self.1.heading)
        }
    }

    /// The location description.
    pub fn description(&self) -> Option<&str> {
        if self.1.description.is_empty() {
            None
        } else {
            Some(&self.1.description)
        }
    }

    /// The latitude, in degrees.
    pub fn latitude(&self) -> f64 {
        self.1.latitude
    }

    /// The longitude, in degrees.
    pub fn longitude(&self) -> f64 {
        self.1.longitude
    }

    /// The timestamp when the location was retrieved.
    pub fn timestamp(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.1.timestamp.0)
    }
}

impl Debug for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Location")
            .field("accuracy", &self.accuracy())
            .field("altitude", &self.altitude())
            .field("speed", &self.speed())
            .field("heading", &self.heading())
            .field("description", &self.description())
            .field("latitude", &self.latitude())
            .field("longitude", &self.longitude())
            .field("timestamp", &self.timestamp())
            .finish()
    }
}

#[derive(Debug, SerializeDict, DeserializeDict, Type)]
#[zvariant(signature = "dict")]
struct LocationInner {
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

/// The interface lets sandboxed applications query basic information about the
/// location.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Location`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Location).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Location")]
pub struct LocationProxy<'a>(zbus::Proxy<'a>);

impl<'a> LocationProxy<'a> {
    /// Create a new instance of [`LocationProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<LocationProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Location")?
            .path(PATH)?
            .destination(DESTINATION)?
            .build()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::Proxy<'_> {
        &self.0
    }

    /// Signal emitted when the user location is updated.
    ///
    /// # Specifications
    ///
    /// See also [`LocationUpdated`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-signal-org-freedesktop-portal-Location.LocationUpdated).
    #[doc(alias = "LocationUpdated")]
    #[doc(alias = "XdpPortal::location-updated")]
    pub async fn receive_location_updated(&self) -> Result<Location, Error> {
        receive_signal(&self.0, "LocationUpdated").await
    }

    /// Create a location session.
    ///
    /// # Arguments
    ///
    /// * `distance_threshold` - Sets the distance threshold in meters, default
    ///   to `0`.
    /// * `time_threshold` - Sets the time threshold in seconds, default to `0`.
    /// * `accuracy` - Sets the location accuracy, default to
    ///   [`Accuracy::Exact`].
    ///
    /// # Specifications
    ///
    /// See also [`CreateSession`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Location.CreateSession).
    #[doc(alias = "CreateSession")]
    pub async fn create_session(
        &self,
        distance_threshold: Option<u32>,
        time_threshold: Option<u32>,
        accuracy: Option<Accuracy>,
    ) -> Result<SessionProxy<'a>, Error> {
        let options = CreateSessionOptions::default()
            .distance_threshold(distance_threshold.unwrap_or(0))
            .time_threshold(time_threshold.unwrap_or(0))
            .accuracy(accuracy.unwrap_or(Accuracy::Exact));
        let (path, proxy) = futures::try_join!(
            call_method::<OwnedObjectPath, CreateSessionOptions>(
                &self.0,
                "CreateSession",
                &(options)
            )
            .into_future(),
            SessionProxy::from_unique_name(self.0.connection(), &options.session_handle_token)
                .into_future(),
        )?;
        assert_eq!(proxy.inner().path(), &path.into_inner());
        Ok(proxy)
    }

    /// Start the location session.
    /// An application can only attempt start a session once.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`LocationProxy::create_session`].
    /// * `identifier` - Identifier for the application window.
    ///
    /// # Specifications
    ///
    /// See also [`Start`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Location.Start).
    #[doc(alias = "Start")]
    #[doc(alias = "xdp_portal_location_monitor_start")]
    pub async fn start(
        &self,
        session: &SessionProxy<'_>,
        identifier: &WindowIdentifier,
    ) -> Result<(), Error> {
        let options = SessionStartOptions::default();
        call_basic_response_method(
            &self.0,
            &options.handle_token,
            "Start",
            &(session, &identifier, &options),
        )
        .await
    }
}

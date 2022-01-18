//! **Note** This portal doesn't work for sandboxed applications.
//! # Examples
//!
//! Access a [`Device`](crate::desktop::device::Device)
//!
//! ```rust,no_run
//! use ashpd::desktop::device::{Device, DeviceProxy};
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = DeviceProxy::new(&connection).await?;
//!     proxy.access_device(6879, &[Device::Speakers]).await?;
//!     Ok(())
//! }
//! ```

use serde::{Deserialize, Serialize, Serializer};
use std::{fmt, str::FromStr};
use zbus::zvariant::{DeserializeDict, SerializeDict, Signature, Type};

use super::{HandleToken, DESTINATION, PATH};
use crate::{helpers::call_basic_response_method, Error};

#[derive(SerializeDict, DeserializeDict, Type, Clone, Debug, Default)]
/// Specified options for a [`DeviceProxy::access_device`] request.
#[zvariant(signature = "dict")]
struct AccessDeviceOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
/// The possible device to request access to.
pub enum Device {
    /// A microphone.
    Microphone,
    /// Speakers.
    Speakers,
    /// A Camera.
    Camera,
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Microphone => write!(f, "Microphone"),
            Self::Speakers => write!(f, "Speakers"),
            Self::Camera => write!(f, "Camera"),
        }
    }
}

impl AsRef<str> for Device {
    fn as_ref(&self) -> &str {
        match self {
            Self::Microphone => "Microphone",
            Self::Speakers => "Speakers",
            Self::Camera => "Camera",
        }
    }
}

impl From<Device> for &'static str {
    fn from(d: Device) -> Self {
        match d {
            Device::Microphone => "Microphone",
            Device::Speakers => "Speakers",
            Device::Camera => "Camera",
        }
    }
}

impl FromStr for Device {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Microphone" | "microphone" => Ok(Device::Microphone),
            "Speakers" | "speakers" => Ok(Device::Speakers),
            "Camera" | "camera" => Ok(Device::Camera),
            _ => Err(Error::ParseError(
                "Failed to parse device, invalid value".to_string(),
            )),
        }
    }
}

impl Serialize for Device {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string().to_lowercase())
    }
}

impl Type for Device {
    fn signature() -> Signature<'static> {
        String::signature()
    }
}

/// The interface lets services ask if an application should get access to
/// devices such as microphones, speakers or cameras. Not a portal in the strict
/// sense, since the API is not directly accessible to applications inside the
/// sandbox.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Device`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Device).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Device")]
pub struct DeviceProxy<'a>(zbus::Proxy<'a>);

impl<'a> DeviceProxy<'a> {
    /// Create a new instance of [`DeviceProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<DeviceProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Device")?
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

    /// Asks for access to a device.
    ///
    /// # Arguments
    ///
    /// * `pid` - The pid of the application on whose behalf the request is
    ///   made.
    /// * `devices` - A list of devices to request access to.
    ///
    /// *Note* Asking for multiple devices at the same time may or may not be supported
    ///
    /// # Specifications
    ///
    /// See also [`AccessDevice`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Device.AccessDevice).
    #[doc(alias = "AccessDevice")]
    pub async fn access_device(&self, pid: u32, devices: &[Device]) -> Result<(), Error> {
        let options = AccessDeviceOptions::default();
        call_basic_response_method(
            self.inner(),
            &options.handle_token,
            "AccessDevice",
            &(pid, devices, &options),
        )
        .await
    }
}

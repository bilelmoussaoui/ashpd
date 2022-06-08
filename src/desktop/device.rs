//! **Note** This portal doesn't work for sandboxed applications.
//! # Examples
//!
//! Access a [`Device`](crate::desktop::device::DeviceKind)
//!
//! ```rust,no_run
//! use ashpd::desktop::device::{DeviceKind, Device};
//!
//! async fn run() -> ashpd::Result<()> {
//!     let proxy = Device::new().await?;
//!     proxy.access_device(6879, &[DeviceKind::Speakers]).await?;
//!     Ok(())
//! }
//! ```

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use zbus::zvariant::{DeserializeDict, SerializeDict, Type};

use super::{HandleToken, DESTINATION, PATH};
use crate::{
    helpers::{call_basic_response_method, session_connection},
    Error,
};

#[derive(SerializeDict, DeserializeDict, Type, Clone, Debug, Default)]
/// Specified options for a [`Device::access_device`] request.
#[zvariant(signature = "dict")]
struct AccessDeviceOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Type)]
#[zvariant(signature = "s")]
#[serde(rename_all = "lowercase")]
/// The possible device to request access to.
pub enum DeviceKind {
    /// A microphone.
    Microphone,
    /// Speakers.
    Speakers,
    /// A Camera.
    Camera,
}

impl fmt::Display for DeviceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Microphone => write!(f, "Microphone"),
            Self::Speakers => write!(f, "Speakers"),
            Self::Camera => write!(f, "Camera"),
        }
    }
}

impl AsRef<str> for DeviceKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Microphone => "Microphone",
            Self::Speakers => "Speakers",
            Self::Camera => "Camera",
        }
    }
}

impl From<DeviceKind> for &'static str {
    fn from(d: DeviceKind) -> Self {
        match d {
            DeviceKind::Microphone => "Microphone",
            DeviceKind::Speakers => "Speakers",
            DeviceKind::Camera => "Camera",
        }
    }
}

impl FromStr for DeviceKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Microphone" | "microphone" => Ok(Self::Microphone),
            "Speakers" | "speakers" => Ok(Self::Speakers),
            "Camera" | "camera" => Ok(Self::Camera),
            _ => Err(Error::ParseError("Failed to parse device, invalid value")),
        }
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
pub struct Device<'a>(zbus::Proxy<'a>);

impl<'a> Device<'a> {
    /// Create a new instance of [`Device`].
    pub async fn new() -> Result<Device<'a>, Error> {
        let connection = session_connection().await?;

        let proxy = zbus::ProxyBuilder::new_bare(&connection)
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
    /// *Note* Asking for multiple devices at the same time may or may not be
    /// supported
    ///
    /// # Specifications
    ///
    /// See also [`AccessDevice`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Device.AccessDevice).
    #[doc(alias = "AccessDevice")]
    pub async fn access_device(&self, pid: u32, devices: &[DeviceKind]) -> Result<(), Error> {
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

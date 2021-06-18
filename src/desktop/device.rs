//! # Examples
//!
//! Access a [`Device`]
//!
//! ```rust,no_run
//! use ashpd::desktop::device::{AccessDeviceOptions, Device, DeviceProxy};
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = DeviceProxy::new(&connection).await?;
//!     let request = proxy.access_device(6879, &[Device::Speakers], AccessDeviceOptions::default()).await?;
//!
//!     let response = request.receive_response::<ashpd::BasicResponse>().await?;
//!     println!("{:#?}", response);
//!     Ok(())
//! }
//! ```
//! [`Device`]: ./enum.Device.html
use serde::{Deserialize, Serialize, Serializer};
use strum_macros::{AsRefStr, EnumString, IntoStaticStr, ToString};
use zvariant::Signature;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{Error, HandleToken, RequestProxy};

#[derive(SerializeDict, DeserializeDict, TypeDict, Clone, Debug, Default)]
/// Specified options for a `access_device` request.
pub struct AccessDeviceOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
}

impl AccessDeviceOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }
}

#[derive(
    Debug, Clone, Copy, Deserialize, EnumString, AsRefStr, IntoStaticStr, ToString, PartialEq, Eq,
)]
#[strum(serialize_all = "lowercase")]
/// The possible device to request access to.
pub enum Device {
    /// A microphone.
    Microphone,
    /// Speakers.
    Speakers,
    /// A Camera.
    Camera,
}

impl zvariant::Type for Device {
    fn signature() -> Signature<'static> {
        String::signature()
    }
}

impl Serialize for Device {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        String::serialize(&self.to_string(), serializer)
    }
}

/// The interface lets services ask if an application should get access to
/// devices such as microphones, speakers or cameras. Not a portal in the strict
/// sense, since the API is not directly accessible to applications inside the
/// sandbox.
pub struct DeviceProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> DeviceProxy<'a> {
    pub async fn new(connection: &zbus::azync::Connection) -> Result<DeviceProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Device")
            .path("/org/freedesktop/portal/desktop")?
            .destination("org.freedesktop.portal.Desktop")
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Asks for access to a device.
    ///
    /// # Arguments
    ///
    /// * `pid` - The pid of the application on whose behalf the request is
    ///   made.
    /// * `devices` - A list of devices to request access to.
    /// * `options` - A [`AccessDeviceOptions`].
    ///
    /// [`AccessDeviceOptions`]: ./struct.AccessDeviceOptions.html
    pub async fn access_device(
        &self,
        pid: u32,
        devices: &[Device],
        options: AccessDeviceOptions,
    ) -> Result<RequestProxy<'_>, Error> {
        let path: zvariant::OwnedObjectPath = self
            .0
            .call_method("AccessDevice", &(pid, devices, options))
            .await?
            .body()?;
        RequestProxy::new(self.0.connection(), path).await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        self.0
            .get_property::<u32>("version")
            .await
            .map_err(From::from)
    }
}

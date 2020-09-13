//! # Examples
//!
//! Access a [`Device`]
//!
//! ```no_run
//! use libportal::desktop::device::{DeviceProxy, AccessDeviceOptions, Device};
//! use libportal::{RequestProxy, BasicResponse as Basic, Response};
//! use zbus::fdo::Result;
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = DeviceProxy::new(&connection)?;
//!     let request_handle = proxy.access_device(
//!         6879,
//!         &[Device::Speakers],
//!         AccessDeviceOptions::default(),
//!     )?;
//!
//!     let request = RequestProxy::new(&connection, &request_handle)?;
//!     request.on_response(|response: Response<Basic>| {
//!         println!("{}", response.is_ok());
//!     })?;
//!     Ok(())
//! }
//! ```
//! [`Device`]: ./enum.Device.html
use crate::HandleToken;
use serde::{Deserialize, Serialize, Serializer};
use strum_macros::{AsRefStr, EnumString, IntoStaticStr, ToString};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{OwnedObjectPath, Signature};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a device access request.
pub struct AccessDeviceOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<HandleToken>,
}

impl AccessDeviceOptions {
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }
}

#[derive(
    Debug, Clone, Deserialize, EnumString, AsRefStr, IntoStaticStr, ToString, PartialEq, Eq,
)]
#[strum(serialize_all = "lowercase")]
pub enum Device {
    Microphone,
    Speakers,
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

#[dbus_proxy(
    interface = "org.freedesktop.portal.Device",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets services ask if an application should get access to devices such as microphones, speakers or cameras.
/// Not a portal in the strict sense, since the API is not directly accessible to applications inside the sandbox.
trait Device {
    /// Asks for access to a device.
    ///
    /// Returns a [`RequestProxy`] object path..
    ///
    /// # Arguments
    ///
    /// * `pid` - The pid of the application on whose behalf the request is made
    /// * `devices` - A list of devices to request access to.
    /// * `options` - A [`AccessDeviceOptions`].
    ///
    /// [`AccessDeviceOptions`]: ./struct.AccessDeviceOptions.html
    /// [`RequestProxy`]: ../../request/struct.RequestProxy.html
    fn access_device(
        &self,
        pid: u32,
        devices: &[Device],
        options: AccessDeviceOptions,
    ) -> Result<OwnedObjectPath>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

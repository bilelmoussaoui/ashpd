use zbus::{dbus_proxy, fdo::Result};
use zvariant::OwnedObjectPath;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a device access request.
pub struct DeviceAccessOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
}

#[derive(Debug, Default)]
pub struct DeviceAccessOptionsBuilder {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
}

impl DeviceAccessOptionsBuilder {
    pub fn handle_token(mut self, handle_token: &str) -> Self {
        self.handle_token = Some(handle_token.to_string());
        self
    }

    pub fn build(self) -> DeviceAccessOptions {
        DeviceAccessOptions {
            handle_token: self.handle_token
        }
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
    /// # Arguments
    ///
    /// * `pid` - The pid of the application on whose behalf the request is made
    /// * `devices` - A list of devices to request access to. Supported values are 'microphone', 'speakers', 'camera'. Asking for multiple devices at the same time may or may not be supported
    ///     FIXME: convert to an enum
    /// * `options` - [`DeviceAccessOptions`]
    ///
    /// [`DeviceAccessOptions`]: ./struct.DeviceAccessOptions.html
    fn access_device(
        &self,
        pid: u32,
        devices: &[&str],
        options: DeviceAccessOptions,
    ) -> Result<OwnedObjectPath>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

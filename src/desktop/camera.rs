use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Camera",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Camera {
    /// AccessCamera method
    fn access_camera(&self, options: HashMap<&str, zvariant::Value>) -> Result<String>;

    /// OpenPipeWireRemote method
    fn open_pipe_wire_remote(
        &self,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<std::os::unix::io::RawFd>;

    /// IsCameraPresent property
    #[dbus_proxy(property)]
    fn is_camera_present(&self) -> Result<bool>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

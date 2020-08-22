use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Wallpaper",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Wallpaper {
    /// SetWallpaperFile method
    fn set_wallpaper_file(
        &self,
        parent_window: &str,
        fd: std::os::unix::io::RawFd,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// SetWallpaperURI method
    fn set_wallpaper_uri(
        &self,
        parent_window: &str,
        uri: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

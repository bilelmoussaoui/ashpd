use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Wallpaper",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications set the user's desktop background picture.
trait Wallpaper {
    /// Sets the lockscreen, background or both wallapers from a file descriptor
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `fd` - The wallapaper file description
    /// * `options` - A hashmap
    ///
    ///     * `show-preview` -  boolean whether to show a preview of the picture
    ///                 . Note that the portal may decide to show a preview even if this option is not set
    ///     * `set-on` - string where to set the wallpaper.
    ///           : possible values background, locksreen, both
    ///
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn set_wallpaper_file(
        &self,
        parent_window: &str,
        fd: std::os::unix::io::RawFd,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// Sets the lockscreen, background or both wallapers from an URI
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `uri` - The wallapaper URI
    /// * `options` - A hashmap
    ///
    ///     * `show-preview` -  boolean whether to show a preview of the picture
    ///                 . Note that the portal may decide to show a preview even if this option is not set
    ///     * `set-on` - string where to set the wallpaper.
    ///           : possible values background, locksreen, both
    ///
    /// [`Request`]: ../request/struct.RequestProxy.html
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

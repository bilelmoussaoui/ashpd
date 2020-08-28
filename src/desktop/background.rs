use crate::WindowIdentifier;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::OwnedObjectPath;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options for a background request.
pub struct BackgroundOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
    /// User-visible reason for the request.
    pub reason: String,
    /// `true` if the app also wants to be started automatically at login.
    pub autostart: bool,
    /// if `true`, use D-Bus activation for autostart.
    pub dbus_activatable: bool,
    //Commandline to use when autostarting at login. If this is not specified, the Exec line from the desktop file will be used.
    //commandline:  Vec<String>,
}
#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Result returned by the response signal after a background request.
pub struct BackgroundResult {
    /// if the application is allowed to run in the background
    background: bool,
    /// if the application is will be autostarted.
    autostart: bool,
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Background",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications request that the application
/// is allowed to run in the background or started automatically when the user logs in.
trait Background {
    /// Requests that the application is allowed to run in the background.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `options` - [`BackgroundOptions`]
    ///
    /// [`BackgroundOptions`]: ./struct.BackgroundOptions.html
    fn request_background(
        &self,
        parent_window: WindowIdentifier,
        options: BackgroundOptions,
    ) -> Result<OwnedObjectPath>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

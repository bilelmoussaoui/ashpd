use crate::WindowIdentifier;
use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
pub struct UpdateOptions {}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Flatpak",
    default_service = "org.freedesktop.portal.Flatpak.UpdateMonitor",
    default_path = "/org/freedesktop/portal/Flatpak"
)]
/// The interface exposes some interactions with Flatpak on the host to the sandbox.
/// For example, it allows you to restart the applications or start a more sandboxed instance.
trait UpdateMonitor {
    ///  Ends the update monitoring and cancels any ongoing installation.
    fn close(&self) -> Result<()>;

    /// Asks to install an update of the calling app.
    ///
    /// Note that updates are only allowed if the new version
    /// has the same permissions (or less) than the currently installed version
    fn update(&self, parent_window: WindowIdentifier, options: UpdateOptions) -> Result<()>;

    // FIXME signal
    // fn update_available(&self, update_info: HashMap<&str, Value>);

    // FIXME signal
    // fn progress(&self, info: HashMap<&str, Value>);
}

use crate::WindowIdentifier;
use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified the options for a get user information request.
pub struct UserInfoOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
    /// Shown in the dialog to explain why the information is needed.
    pub reason: String,
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Account",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications query basic information about the user,
/// like his name and avatar photo.
///
/// The portal backend will present the user with a dialog to confirm which (if any) information to share.
trait Account {
    /// Gets information about the user.
    ///
    /// # Arguments
    ///
    /// * `window` - Identifier for the window
    /// * `options` - A [`UserInfoOptions`]
    ///
    /// [`UserInfoOptions`]: ./struct.UserInfoOptions.html
    fn get_user_information(
        &self,
        window: WindowIdentifier,
        options: UserInfoOptions,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

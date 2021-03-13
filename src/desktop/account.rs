//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::account::{AccountProxy, UserInfo, UserInfoOptions};
//! use ashpd::{Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = AccountProxy::new(&connection)?;
//!     let request = proxy.get_user_information(
//!         WindowIdentifier::default(),
//!         UserInfoOptions::default().reason("Fractal would like access to your information"),
//!     )?;
//!     request.connect_response(|response: Response<UserInfo>| {
//!         let user_info = response.unwrap();
//!
//!         println!("{}", user_info.id);
//!         println!("{}", user_info.name);
//!         println!("{}", user_info.image);
//!
//!         Ok(())
//!     })?;
//!     Ok(())
//! }
//! ```
use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{AsyncRequestProxy, HandleToken, RequestProxy, WindowIdentifier};

#[derive(SerializeDict, DeserializeDict, TypeDict, Clone, Debug, Default)]
/// The possible options for a get user information request.
pub struct UserInfoOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
    /// Shown in the dialog to explain why the information is needed.
    reason: Option<String>,
}

impl UserInfoOptions {
    /// Sets a user-visible reason for the request.
    pub fn reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }

    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }
}

#[derive(Debug, SerializeDict, DeserializeDict, Clone, TypeDict)]
/// The response of a `get_user_information` request.
pub struct UserInfo {
    /// User identifier.
    pub id: String,
    /// User name.
    pub name: String,
    /// User image uri.
    pub image: String,
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Account",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications query basic information about the
/// user, like his name and avatar photo.
///
/// The portal backend will present the user with a dialog to confirm which (if
/// any) information to share.
trait Account {
    /// Gets information about the user.
    ///
    /// # Arguments
    ///
    /// * `window` - Identifier for the window.
    /// * `options` - A [`UserInfoOptions`].
    ///
    /// [`UserInfoOptions`]: ./struct.UserInfoOptions.html
    #[dbus_proxy(object = "Request")]
    fn get_user_information(&self, window: WindowIdentifier, options: UserInfoOptions);

    /// The version of this DBus interface.
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

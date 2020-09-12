//! # Examples
//!
//! ```no_run
//! use libportal::desktop::account::{AccountProxy, UserInfoOptions, UserInfo};
//! use libportal::{RequestProxy, Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = AccountProxy::new(&connection)?;
//!     let request_handle = proxy.get_user_information(
//!         WindowIdentifier::default(),
//!         UserInfoOptions::default()
//!             .reason("Fractal would like access to your information"),
//!     )?;
//!     let req = RequestProxy::new(&connection, &request_handle)?;
//!     req.on_response(|response: Response<UserInfo>| {
//!         if let Ok(user_info) = response {
//!             println!("{}", user_info.id);
//!             println!("{}", user_info.name);
//!             println!("{}", user_info.image);
//!         }
//!     })?;
//!     Ok(())
//! }
//!```
use crate::WindowIdentifier;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::OwnedObjectPath;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified the options for a get user information request.
pub struct UserInfoOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
    /// Shown in the dialog to explain why the information is needed.
    pub reason: Option<String>,
}

impl UserInfoOptions {
    pub fn reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }

    pub fn handle_token(mut self, handle_token: &str) -> Self {
        self.handle_token = Some(handle_token.to_string());
        self
    }
}

#[derive(Debug, SerializeDict, DeserializeDict, TypeDict)]
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
/// The interface lets sandboxed applications query basic information about the user,
/// like his name and avatar photo.
///
/// The portal backend will present the user with a dialog to confirm which (if any) information to share.
trait Account {
    /// Gets information about the user.
    ///
    /// Returns a [`RequestProxy`] handle.
    ///
    /// # Arguments
    ///
    /// * `window` - Identifier for the window
    /// * `options` - A [`UserInfoOptions`]
    ///
    /// [`UserInfoOptions`]: ./struct.UserInfoOptions.html
    /// [`RequestProxy`]: ../../request/struct.RequestProxy.html
    fn get_user_information(
        &self,
        window: WindowIdentifier,
        options: UserInfoOptions,
    ) -> Result<OwnedObjectPath>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

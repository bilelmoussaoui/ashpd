//! # Examples
//!
//! ```rust,no_run
//! use ashpd::{desktop::account, Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! async fn run() -> Result<()> {
//!     let identifier = WindowIdentifier::default();
//!     if let Response::Ok(user_info) =
//!         account::get_user_information(identifier, "App would like to access user information").await?
//!     {
//!         println!("Name: {}", user_info.name);
//!         println!("ID: {}", user_info.id);
//!     }
//!     Ok(())
//! }
//! ```
use std::sync::Arc;

use futures::{lock::Mutex, FutureExt};
use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{AsyncRequestProxy, HandleToken, RequestProxy, Response, WindowIdentifier};

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

/// Get the user information
///
/// An async wrapper around the `AsyncAccountProxy::get_user_information`
/// function.
pub async fn get_user_information(
    window_identifier: WindowIdentifier,
    reason: &str,
) -> zbus::Result<Response<UserInfo>> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncAccountProxy::new(&connection)?;
    let request = proxy
        .get_user_information(
            window_identifier,
            UserInfoOptions::default().reason(&reason),
        )
        .await?;

    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Arc::new(Mutex::new(Some(sender)));
    let signal_id = request
        .connect_response(move |response: Response<UserInfo>| {
            let s = sender.clone();
            async move {
                if let Some(m) = s.lock().await.take() {
                    let _ = m.send(response);
                }
                Ok(())
            }
            .boxed()
        })
        .await?;

    while request.next_signal().await?.is_some() {}
    request.disconnect_signal(signal_id).await?;

    let user_information = receiver.await.unwrap();
    Ok(user_information)
}

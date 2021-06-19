//! # Examples
//!
//! ```rust,no_run
//! use ashpd::{
//!     desktop::account::{AccountProxy, UserInfoOptions},
//!     WindowIdentifier,
//! };
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let identifier = WindowIdentifier::default();
//!     let connection = zbus::azync::Connection::new_session().await?;
//!
//!     let proxy = AccountProxy::new(&connection).await?;
//!     let user_info = proxy
//!         .user_information(
//!             identifier,
//!             UserInfoOptions::default().reason("App would like to access user information"),
//!         )
//!         .await?;
//!
//!     println!("Name: {}", user_info.name);
//!     println!("ID: {}", user_info.id);
//!
//!     Ok(())
//! }
//! ```

use crate::{
    helpers::{call_request_method, property},
    Error, HandleToken, WindowIdentifier,
};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Clone, Debug, Default)]
/// Specified options for a [`AccountProxy::user_information`] request.
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
/// The response of a [`AccountProxy::user_information`] request.
pub struct UserInfo {
    /// User identifier.
    pub id: String,
    /// User name.
    pub name: String,
    /// User image uri.
    pub image: String,
}

/// The interface lets sandboxed applications query basic information about the
/// user, like his name and avatar photo.
///
/// The portal backend will present the user with a dialog to confirm which (if
/// any) information to share.
#[derive(Debug)]
pub struct AccountProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> AccountProxy<'a> {
    /// Create a new instance of [`AccountProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<AccountProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Account")
            .path("/org/freedesktop/portal/desktop")?
            .destination("org.freedesktop.portal.Desktop")
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Gets information about the user.
    ///
    /// # Arguments
    ///
    /// * `window` - Identifier for the window.
    /// * `options` - A [`UserInfoOptions`].
    pub async fn user_information(
        &self,
        window: WindowIdentifier,
        options: UserInfoOptions,
    ) -> Result<UserInfo, Error> {
        call_request_method(&self.0, "GetUserInformation", &(window, options)).await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}

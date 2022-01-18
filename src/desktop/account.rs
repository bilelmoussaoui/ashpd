//! # Examples
//!
//! ```rust, no_run
//! use ashpd::desktop::account;
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let user_info = account::user_information(
//!         &WindowIdentifier::default(),
//!         "App would like to access user information",
//!     ).await?;
//!
//!     println!("Name: {}", user_info.name());
//!     println!("ID: {}", user_info.id());
//!
//!     Ok(())
//! }
//! ```
//!
//! Or by using the Proxy directly
//!
//! ```rust,no_run
//! use ashpd::desktop::account::AccountProxy;
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!
//!     let proxy = AccountProxy::new(&connection).await?;
//!     let user_info = proxy
//!         .user_information(
//!             &WindowIdentifier::default(),
//!             "App would like to access user information",
//!         )
//!         .await?;
//!
//!     println!("Name: {}", user_info.name());
//!     println!("ID: {}", user_info.id());
//!
//!     Ok(())
//! }
//! ```

use zbus::zvariant::{DeserializeDict, SerializeDict, Type};

use super::{HandleToken, DESTINATION, PATH};
use crate::{helpers::call_request_method, Error, WindowIdentifier};

#[derive(SerializeDict, DeserializeDict, Type, Clone, Debug, Default)]
/// Specified options for a [`AccountProxy::user_information`] request.
#[zvariant(signature = "dict")]
struct UserInfoOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// Shown in the dialog to explain why the information is needed.
    reason: Option<String>,
}

impl UserInfoOptions {
    /// Sets a user-visible reason for the request.
    pub fn reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }
}

#[derive(Debug, SerializeDict, DeserializeDict, Clone, Type)]
/// The response of a [`AccountProxy::user_information`] request.
#[zvariant(signature = "dict")]
pub struct UserInfo {
    /// User identifier.
    id: String,
    /// User name.
    name: String,
    /// User image uri.
    image: String,
}

impl UserInfo {
    /// User identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// User name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// User image uri.
    pub fn image(&self) -> &str {
        &self.image
    }
}

/// The interface lets sandboxed applications query basic information about the
/// user, like his name and avatar photo.
///
/// The portal backend will present the user with a dialog to confirm which (if
/// any) information to share.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Account`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Account).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Account")]
pub struct AccountProxy<'a>(zbus::Proxy<'a>);

impl<'a> AccountProxy<'a> {
    /// Create a new instance of [`AccountProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<AccountProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Account")?
            .path(PATH)?
            .destination(DESTINATION)?
            .build()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::Proxy<'_> {
        &self.0
    }

    /// Gets information about the user.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the window.
    /// * `reason` - A user-visible reason for the request.
    #[doc(alias = "GetUserInformation")]
    pub async fn user_information(
        &self,
        identifier: &WindowIdentifier,
        reason: &str,
    ) -> Result<UserInfo, Error> {
        let options = UserInfoOptions::default().reason(reason);
        call_request_method(
            self.inner(),
            &options.handle_token,
            "GetUserInformation",
            &(&identifier, &options),
        )
        .await
    }
}

#[doc(alias = "xdp_portal_get_user_information")]
#[doc(alias = "get_user_information")]
/// A handy wrapper around [`AccountProxy::user_information`].
pub async fn user_information(
    identifier: &WindowIdentifier,
    reason: &str,
) -> Result<UserInfo, Error> {
    let connection = zbus::Connection::session().await?;
    let proxy = AccountProxy::new(&connection).await?;
    proxy.user_information(identifier, reason).await
}

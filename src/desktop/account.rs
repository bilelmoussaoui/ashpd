//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::account::AccountProxy;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!
//!     let proxy = AccountProxy::new(&connection).await?;
//!     let user_info = proxy
//!         .user_information(
//!             Default::default(),
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

use crate::{
    helpers::{call_request_method, property},
    Error, WindowIdentifier,
};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use super::{HandleToken, DESTINATION, PATH};

#[derive(SerializeDict, DeserializeDict, TypeDict, Clone, Debug, Default)]
/// Specified options for a [`AccountProxy::user_information`] request.
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

#[derive(Debug, SerializeDict, DeserializeDict, Clone, TypeDict)]
/// The response of a [`AccountProxy::user_information`] request.
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
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Account")]
pub struct AccountProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> AccountProxy<'a> {
    /// Create a new instance of [`AccountProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<AccountProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Account")
            .path(PATH)?
            .destination(DESTINATION)
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Gets information about the user.
    ///
    /// # Arguments
    ///
    /// * `window` - Identifier for the window.
    /// * `reason` - A user-visible reason for the request.
    #[doc(alias = "GetUserInformation")]
    pub async fn user_information(
        &self,
        window: WindowIdentifier,
        reason: &str,
    ) -> Result<UserInfo, Error> {
        let options = UserInfoOptions::default().reason(reason);
        call_request_method(
            &self.0,
            &options.handle_token,
            "GetUserInformation",
            &(window, &options),
        )
        .await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}

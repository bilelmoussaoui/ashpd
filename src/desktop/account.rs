//! The interface lets sandboxed applications query basic information about the
//! user, like his name and avatar photo.
//!
//! The portal backend will present the user with a dialog to confirm which (if
//! any) information to share.
//!
//! Wrapper of the DBus interface: [`org.freedesktop.portal.Account`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Account).
//!
//! # Examples
//!
//! ```rust, no_run
//! use ashpd::desktop::account::UserInformationRequest;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let response = UserInformationRequest::default()
//!         .reason("App would like to access user information")
//!         .build()
//!         .await?;
//!
//!     println!("Name: {}", response.name());
//!     println!("ID: {}", response.id());
//!
//!     Ok(())
//! }
//! ```

use zbus::zvariant::{DeserializeDict, SerializeDict, Type};

use super::{HandleToken, DESTINATION, PATH};
use crate::{
    helpers::{call_request_method, session_connection},
    Error, WindowIdentifier,
};

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct UserInformationOptions {
    handle_token: HandleToken,
    reason: Option<String>,
}

#[derive(Debug, DeserializeDict, Type)]
/// The response of a [`UserInformationRequest`] request.
#[zvariant(signature = "dict")]
pub struct UserInformationResponse {
    /// User identifier.
    id: String,
    /// User name.
    name: String,
    /// User image uri.
    image: url::Url,
}

impl UserInformationResponse {
    /// User identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// User name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// User image uri.
    pub fn image(&self) -> &url::Url {
        &self.image
    }

    /// Creates a new builder-pattern struct instance to construct
    /// [`UserInformationResponse`].
    ///
    /// This method returns an instance of [`UserInformationRequest`].
    pub fn builder() -> UserInformationRequest {
        UserInformationRequest::default()
    }
}

#[doc(alias = "org.freedesktop.portal.Account")]
struct AccountProxy<'a>(zbus::Proxy<'a>);

impl<'a> AccountProxy<'a> {
    /// Create a new instance of [`AccountProxy`].
    pub async fn new() -> Result<AccountProxy<'a>, Error> {
        let connection = session_connection().await?;
        let proxy = zbus::ProxyBuilder::new_bare(&connection)
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
        options: UserInformationOptions,
    ) -> Result<UserInformationResponse, Error> {
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
#[derive(Debug, Default)]
/// A [builder-pattern] type to construct [`UserInformationResponse`].
///
/// [builder-pattern]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
pub struct UserInformationRequest {
    reason: Option<String>,
    identifier: WindowIdentifier,
}

impl UserInformationRequest {
    #[must_use]
    /// Sets a user-visible reason for the request.
    pub fn reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_owned());
        self
    }

    #[must_use]
    /// Sets a window identifier.
    pub fn identifier(mut self, identifier: WindowIdentifier) -> Self {
        self.identifier = identifier;
        self
    }

    /// Build the [`UserInformationResponse`].
    pub async fn build(self) -> Result<UserInformationResponse, Error> {
        let proxy = AccountProxy::new().await?;
        let options = UserInformationOptions {
            reason: self.reason,
            ..Default::default()
        };
        proxy.user_information(&self.identifier, options).await
    }
}

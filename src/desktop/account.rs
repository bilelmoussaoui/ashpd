//! Access to the current logged user information such as the id, name
//! or their avatar uri.
//!
//! Wrapper of the DBus interface: [`org.freedesktop.portal.Account`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Account).
//!
//! ### Examples
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
    id: String,
    name: String,
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

struct AccountProxy<'a>(zbus::Proxy<'a>);

impl<'a> AccountProxy<'a> {
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

    pub fn inner(&self) -> &zbus::Proxy<'_> {
        &self.0
    }

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
#[doc(alias = "org.freedesktop.portal.Account")]
#[derive(Debug, Default)]
/// A [builder-pattern] type to construct [`UserInformationResponse`].
///
/// [builder-pattern]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
pub struct UserInformationRequest {
    options: UserInformationOptions,
    identifier: WindowIdentifier,
}

impl UserInformationRequest {
    #[must_use]
    /// Sets a user-visible reason for the request.
    pub fn reason<'a>(mut self, reason: impl Into<Option<&'a str>>) -> Self {
        self.options.reason = reason.into().map(ToOwned::to_owned);
        self
    }

    #[must_use]
    /// Sets a window identifier.
    pub fn identifier(mut self, identifier: impl Into<Option<WindowIdentifier>>) -> Self {
        self.identifier = identifier.into().unwrap_or_default();
        self
    }

    /// Build the [`UserInformationResponse`].
    pub async fn build(self) -> Result<UserInformationResponse, Error> {
        let proxy = AccountProxy::new().await?;
        proxy.user_information(&self.identifier, self.options).await
    }
}

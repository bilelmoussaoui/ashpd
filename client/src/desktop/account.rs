//! Access to the current logged user information such as the id, name
//! or their avatar uri.
//!
//! Wrapper of the DBus interface: [`org.freedesktop.portal.Account`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Account.html).
//!
//! ### Examples
//!
//! ```rust, no_run
//! use ashpd::desktop::account::UserInformation;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let response = UserInformation::request()
//!         .reason("App would like to access user information")
//!         .send()
//!         .await?
//!         .response()?;
//!
//!     println!("Name: {}", response.name());
//!     println!("ID: {}", response.id());
//!
//!     Ok(())
//! }
//! ```

use zbus::zvariant::{DeserializeDict, SerializeDict, Type};

use super::HandleToken;
use crate::{
    Error, WindowIdentifier, desktop::request::Request, proxy::Proxy,
    window_identifier::MaybeWindowIdentifierExt,
};

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct UserInformationOptions {
    handle_token: HandleToken,
    reason: Option<String>,
}

#[derive(Debug, DeserializeDict, SerializeDict, Type)]
/// The response of a [`UserInformationRequest`] request.
#[zvariant(signature = "dict")]
pub struct UserInformation {
    id: String,
    name: String,
    image: url::Url,
}

impl UserInformation {
    #[cfg(feature = "backend")]
    #[cfg_attr(docsrs, doc(cfg(feature = "backend")))]
    /// Create a new instance of [`UserInformation`].
    pub fn new(id: &str, name: &str, image: url::Url) -> Self {
        Self {
            id: id.to_owned(),
            name: name.to_owned(),
            image,
        }
    }

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
    /// [`UserInformation`].
    ///
    /// This method returns an instance of [`UserInformationRequest`].
    pub fn request() -> UserInformationRequest {
        UserInformationRequest::default()
    }
}

struct AccountProxy(Proxy<'static>);

impl AccountProxy {
    pub async fn new() -> Result<Self, Error> {
        let proxy = Proxy::new_desktop("org.freedesktop.portal.Account").await?;
        Ok(Self(proxy))
    }

    pub async fn with_connection(connection: zbus::Connection) -> Result<Self, Error> {
        let proxy =
            Proxy::new_desktop_with_connection(connection, "org.freedesktop.portal.Account")
                .await?;
        Ok(Self(proxy))
    }

    #[doc(alias = "GetUserInformation")]
    pub async fn user_information(
        &self,
        identifier: Option<&WindowIdentifier>,
        options: UserInformationOptions,
    ) -> Result<Request<UserInformation>, Error> {
        let identifier = identifier.to_string_or_empty();
        self.0
            .request(
                &options.handle_token,
                "GetUserInformation",
                (&identifier, &options),
            )
            .await
    }
}

impl std::ops::Deref for AccountProxy {
    type Target = zbus::Proxy<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[doc(alias = "xdp_portal_get_user_information")]
#[doc(alias = "org.freedesktop.portal.Account")]
#[derive(Debug, Default)]
/// A [builder-pattern] type to construct [`UserInformation`].
///
/// [builder-pattern]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
pub struct UserInformationRequest {
    options: UserInformationOptions,
    identifier: Option<WindowIdentifier>,
    connection: Option<zbus::Connection>,
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
        self.identifier = identifier.into();
        self
    }

    #[must_use]
    /// Sets a connection to use other than the internal one.
    pub fn connection(mut self, connection: Option<zbus::Connection>) -> Self {
        self.connection = connection;
        self
    }

    /// Build the [`UserInformation`].
    pub async fn send(self) -> Result<Request<UserInformation>, Error> {
        let proxy = if let Some(connection) = self.connection {
            AccountProxy::with_connection(connection).await?
        } else {
            AccountProxy::new().await?
        };
        proxy
            .user_information(self.identifier.as_ref(), self.options)
            .await
    }
}

//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::secret::SecretProxy;
//! use std::fs::File;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = SecretProxy::new(&connection).await?;
//!
//!     let file = File::open("test.txt").unwrap();
//!
//!     let secret = proxy.retrieve_secret(&file, None).await?;
//!
//!     println!("{:#?}", secret);
//!     Ok(())
//! }
//! ```

use std::os::unix::prelude::AsRawFd;

use zbus::zvariant::{DeserializeDict, Fd, SerializeDict, Type};

use super::{DESTINATION, PATH};
use crate::{helpers::call_method, Error};

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`SecretProxy::retrieve_secret`] request.
#[zvariant(signature = "dict")]
struct RetrieveOptions {
    /// A string returned by a previous call to `retrieve_secret`.
    token: Option<String>,
}

impl RetrieveOptions {
    /// Sets the token received on a previous call to
    /// [`SecretProxy::retrieve_secret`].
    #[must_use]
    pub fn token(mut self, token: &str) -> Self {
        self.token = Some(token.to_string());
        self
    }
}

/// The interface lets sandboxed applications retrieve a per-application secret.
/// The secret can then be used for encrypting confidential data inside the
/// sandbox.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Secret`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Secret).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Secret")]
pub struct SecretProxy<'a>(zbus::Proxy<'a>);

impl<'a> SecretProxy<'a> {
    /// Create a new instance of [`SecretProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<SecretProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Secret")?
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

    /// Retrieves a master secret for a sandboxed application.
    ///
    /// # Arguments
    ///
    /// * `fd` - Writable file descriptor for transporting the secret.
    /// * `token` -  A string returned by a previous call to
    ///   [`retrieve_secret()`][`SecretProxy::retrieve_secret`].
    #[doc(alias = "RetrieveSecret")]
    pub async fn retrieve_secret(
        &self,
        fd: &impl AsRawFd,
        token: Option<&str>,
    ) -> Result<String, Error> {
        let options = if let Some(token) = token {
            RetrieveOptions::default().token(token)
        } else {
            RetrieveOptions::default()
        };
        call_method(
            self.inner(),
            "RetrieveSecret",
            &(Fd::from(fd.as_raw_fd()), options),
        )
        .await
    }
}

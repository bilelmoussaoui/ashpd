//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::secret::SecretProxy;
//! use std::fs::File;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = SecretProxy::new(&connection).await?;
//!
//!     let file = File::open("test.txt").unwrap();
//!
//!     let secret = proxy
//!         .retrieve_secret(file, None)
//!         .await?;
//!
//!     println!("{:#?}", secret);
//!     Ok(())
//! }
//! ```

use super::{DESTINATION, PATH};
use crate::{helpers::call_method, Error};
use std::os::unix::prelude::AsRawFd;
use zvariant::Fd;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a [`SecretProxy::retrieve_secret`] request.
struct RetrieveOptions {
    /// A string returned by a previous call to `retrieve_secret`.
    token: Option<String>,
}

impl RetrieveOptions {
    /// Sets the token received on a previous call to [`SecretProxy::retrieve_secret`].
    pub fn token(mut self, token: &str) -> Self {
        self.token = Some(token.to_string());
        self
    }
}

/// The interface lets sandboxed applications retrieve a per-application secret.
/// The secret can then be used for encrypting confidential data inside the
/// sandbox.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Secret`](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html#gdbus-org.freedesktop.portal.Secret).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Secret")]
pub struct SecretProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> SecretProxy<'a> {
    /// Create a new instance of [`SecretProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<SecretProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Secret")
            .path(PATH)?
            .destination(DESTINATION)
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::azync::Proxy<'_> {
        &self.0
    }

    /// Retrieves a master secret for a sandboxed application.
    ///
    /// # Arguments
    ///
    /// * `fd` - Writable file descriptor for transporting the secret.
    /// * `token` -  A string returned by a previous call to [`SecretProxy::retrieve_secret`].
    #[doc(alias = "RetrieveSecret")]
    pub async fn retrieve_secret<F: AsRawFd>(
        &self,
        fd: &F,
        token: Option<&str>,
    ) -> Result<String, Error> {
        let options = if let Some(token) = token {
            RetrieveOptions::default().token(token)
        } else {
            RetrieveOptions::default()
        };
        call_method(
            &self.0,
            "RetrieveSecret",
            &(Fd::from(fd.as_raw_fd()), options),
        )
        .await
    }
}

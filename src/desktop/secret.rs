//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::secret::SecretProxy;
//! use std::io::Read;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = SecretProxy::new(&connection).await?;
//!
//!     let (mut x1, x2) = std::os::unix::net::UnixStream::pair().unwrap();
//!     let secret = proxy.retrieve_secret(&x2).await?;
//!     drop(x2);
//!     let mut buf = Vec::new();
//!     x1.read_to_end(&mut buf);
//!
//!     Ok(())
//! }
//! ```

use std::os::unix::prelude::AsRawFd;

use zbus::zvariant::{Fd, SerializeDict, Type};

use super::{HandleToken, DESTINATION, PATH};
use crate::{helpers::call_basic_response_method, Error};

#[derive(SerializeDict, Type, Debug, Default)]
/// Specified options for a [`SecretProxy::retrieve_secret`] request.
#[zvariant(signature = "dict")]
struct RetrieveOptions {
    handle_token: HandleToken,
    /// A string returned by a previous call to `retrieve_secret`.
    /// TODO: seems to not be used by the portal...
    token: Option<String>,
}

impl RetrieveOptions {
    /// Sets the token received on a previous call to
    /// [`SecretProxy::retrieve_secret`].
    #[must_use]
    #[allow(unused)]
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
    #[doc(alias = "RetrieveSecret")]
    pub async fn retrieve_secret(&self, fd: &impl AsRawFd) -> Result<(), Error> {
        let options = RetrieveOptions::default();
        call_basic_response_method(
            self.inner(),
            &options.handle_token,
            "RetrieveSecret",
            &(Fd::from(fd.as_raw_fd()), &options),
        )
        .await?;
        Ok(())
    }
}

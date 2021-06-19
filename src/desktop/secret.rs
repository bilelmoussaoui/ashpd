//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::secret::{RetrieveOptions, SecretProxy};
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//! use zvariant::Fd;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = SecretProxy::new(&connection).await?;
//!
//!     let file = File::open("test.txt").unwrap();
//!
//!     let secret = proxy.retrieve_secret(Fd::from(file.as_raw_fd()), RetrieveOptions::default()).await?;
//!
//!     println!("{:#?}", secret);
//!     Ok(())
//! }
//! ```

use crate::{
    helpers::{call_method, property},
    Error,
};
use std::os::unix::prelude::AsRawFd;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a retrieve secret request.
pub struct RetrieveOptions {
    /// A string returned by a previous call to `retrieve_secret`.
    token: Option<String>,
}

impl RetrieveOptions {
    /// Sets the token received on a previous call to `retrieve_secret`.
    pub fn token(mut self, token: &str) -> Self {
        self.token = Some(token.to_string());
        self
    }
}

/// The interface lets sandboxed applications retrieve a per-application secret.
/// The secret can then be used for encrypting confidential data inside the
/// sandbox.
pub struct SecretProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> SecretProxy<'a> {
    pub async fn new(connection: &zbus::azync::Connection) -> Result<SecretProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Secret")
            .path("/org/freedesktop/portal/desktop")?
            .destination("org.freedesktop.portal.Desktop")
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Retrieves a master secret for a sandboxed application.
    ///
    /// # Arguments
    ///
    /// * `fd` - Writable file descriptor for transporting the secret.
    /// * `options` - A `RetrieveOptions`.
    ///
    /// [`RetrieveOptions`]: ./struct.RetrieveOptions.html
    pub async fn retrieve_secret<F: AsRawFd + serde::Serialize + zvariant::Type>(
        &self,
        fd: F,
        options: RetrieveOptions,
    ) -> Result<(), Error> {
        call_method(&self.0, "RetrieveSecret", &(fd.as_raw_fd(), options)).await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}

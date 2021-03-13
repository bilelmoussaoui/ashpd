//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::secret::{RetrieveOptions, SecretProxy};
//! use ashpd::{BasicResponse as Basic, Response};
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//! use zbus::{fdo::Result, Connection};
//! use zvariant::Fd;
//!
//! fn main() -> Result<()> {
//!     let connection = Connection::new_session()?;
//!     let proxy = SecretProxy::new(&connection)?;
//!
//!     let file = File::open("test.txt").unwrap();
//!
//!     let request = proxy.retrieve_secret(Fd::from(file.as_raw_fd()), RetrieveOptions::default())?;
//!     request.connect_response(|r: Response<Basic>| {
//!         println!("{:#?}", r);
//!         Ok(())
//!     })?;
//!
//!     Ok(())
//! }
//! ```
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Fd;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{AsyncRequestProxy, RequestProxy};

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

#[dbus_proxy(
    interface = "org.freedesktop.portal.Secret",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications retrieve a per-application secret.
/// The secret can then be used for encrypting confidential data inside the
/// sandbox.
trait Secret {
    /// Retrieves a master secret for a sandboxed application.
    ///
    /// # Arguments
    ///
    /// * `fd` - Writable file descriptor for transporting the secret.
    /// * `options` - A `RetrieveOptions`.
    ///
    /// [`RetrieveOptions`]: ./struct.RetrieveOptions.html
    #[dbus_proxy(object = "Request")]
    fn retrieve_secret(&self, fd: Fd, options: RetrieveOptions);

    /// The version of this DBus interface.
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

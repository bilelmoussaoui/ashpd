//! # Examples
//!
//! ```no_run
//! use ashpd::desktop::secret::{SecretProxy, RetrieveOptions};
//! use ashpd::{RequestProxy, Response, BasicResponse as Basic};
//! use zbus::{fdo::Result, Connection};
//! use zvariant::Fd;
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//!
//! fn main() -> Result<()> {
//!     let connection = Connection::new_session()?;
//!     let proxy = SecretProxy::new(&connection)?;
//!
//!     let file = File::open("test.txt").unwrap();
//!
//!     let handle = proxy.retrieve_secret(Fd::from(file.as_raw_fd()), RetrieveOptions::default())?;
//!     let request = RequestProxy::new(&connection, &handle)?;
//!     request.connect_response(|r: Response<Basic>| {
//!         println!("{:#?}", r);
//!         Ok(())
//!     })?;
//!
//!     Ok(())
//! }
//! ```
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{Fd, OwnedObjectPath};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a retrieve secret request.
pub struct RetrieveOptions {
    /// A string returned by a previous call to `retrieve_secret`
    pub token: Option<String>,
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
/// The secret can then be used for encrypting confidential data inside the sandbox.
trait Secret {
    /// Retrieves a master secret for a sandboxed application.
    ///
    /// Returns a [`RequestProxy`] object path..
    ///
    /// # Arguments
    ///
    /// * `fd` - Writable file descriptor for transporting the secret.
    /// * `options` - A `RetrieveOptions`
    ///
    /// [`RetrieveOptions`]: ./struct.RetrieveOptions.html
    /// [`RequestProxy`]: ../../request/struct.RequestProxy.html
    fn retrieve_secret(&self, fd: Fd, options: RetrieveOptions) -> Result<OwnedObjectPath>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

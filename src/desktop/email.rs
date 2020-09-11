use crate::WindowIdentifier;
use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::OwnedObjectPath;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a compose email request.
pub struct EmailOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
    /// The email address to send to
    pub address: Option<String>,
    // The email adresses to send to
    pub addresses: Option<Vec<String>>,
    // The email adresses to CC
    pub cc: Option<Vec<String>>,
    // The email adresses to BCC
    pub bcc: Option<Vec<String>>,
    /// The subject of the email
    pub subject: Option<String>,
    /// The body of the email
    pub body: Option<String>,
    /// A list of file descriptors of files to attach
    pub attachment_fds: Option<Vec<RawFd>>,
}

impl EmailOptions {
    pub fn address(mut self, address: &str) -> Self {
        self.address = Some(address.to_string());
        self
    }

    pub fn subject(mut self, subject: &str) -> Self {
        self.subject = Some(subject.to_string());
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Email",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications request sending an email.
trait Email {
    /// Presents a window that lets the user compose an email.
    ///
    /// Note that the default email client for the host will need to support mailto: URIs following RFC 2368
    ///
    /// Returns a `Request` handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `options` - [`EmailOptions`]
    ///
    /// [`EmailOptions`]: ./struct.EmailOptions.html
    fn compose_email(
        &self,
        parent_window: WindowIdentifier,
        options: EmailOptions,
    ) -> Result<OwnedObjectPath>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

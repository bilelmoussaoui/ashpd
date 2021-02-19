//! # Examples
//!
//! Compose an email
//!
//! ```rust,no_run
//! use ashpd::desktop::email::{EmailOptions, EmailProxy};
//! use ashpd::{BasicResponse as Basic, RequestProxy, Response, WindowIdentifier};
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//! use zbus::{fdo::Result, Connection};
//! use zvariant::Fd;
//!
//! fn main() -> Result<()> {
//!     let connection = Connection::new_session()?;
//!     let proxy = EmailProxy::new(&connection)?;
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!     let request = proxy.compose_email(
//!         WindowIdentifier::default(),
//!         EmailOptions::default()
//!             .address("test@gmail.com")
//!             .subject("email subject")
//!             .body("the pre-filled email body")
//!             .attach(Fd::from(file.as_raw_fd())),
//!     )?;
//!     request.connect_response(|r: Response<Basic>| {
//!         println!("{}", r.is_ok());
//!         Ok(())
//!     })?;
//!     Ok(())
//! }
//! ```
use crate::{AsyncRequestProxy, HandleToken, RequestProxy, WindowIdentifier};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Fd;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a compose email request.
pub struct EmailOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<HandleToken>,
    /// The email address to send to.
    pub address: Option<String>,
    /// The email addresses to send to.
    pub addresses: Option<Vec<String>>,
    /// The email addresses to CC.
    pub cc: Option<Vec<String>>,
    /// The email addresses to BCC.
    pub bcc: Option<Vec<String>>,
    /// The subject of the email.
    pub subject: Option<String>,
    /// The body of the email.
    pub body: Option<String>,
    /// A list of file descriptors of files to attach.
    pub attachment_fds: Option<Vec<Fd>>,
}

impl EmailOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    /// Sets the email address to send the email to.
    pub fn address(mut self, address: &str) -> Self {
        self.address = Some(address.to_string());
        self
    }

    /// Sets a list of email addresses to send the email to.
    pub fn addresses(mut self, addresses: Vec<String>) -> Self {
        self.addresses = Some(addresses);
        self
    }

    /// Sets a list of email addresses to BCC.
    pub fn bcc(mut self, bcc: Vec<String>) -> Self {
        self.bcc = Some(bcc);
        self
    }

    /// Sets a list of email addresses to CC.
    pub fn cc(mut self, cc: Vec<String>) -> Self {
        self.cc = Some(cc);
        self
    }

    /// Sets the email subject.
    pub fn subject(mut self, subject: &str) -> Self {
        self.subject = Some(subject.to_string());
        self
    }

    /// Sets the email body.
    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }

    /// Attaches a file to the email.
    pub fn attach(mut self, attachment: Fd) -> Self {
        match self.attachment_fds {
            Some(ref mut attachments) => attachments.push(attachment),
            None => {
                self.attachment_fds.replace(vec![attachment]);
            }
        };
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
    /// Returns a [`RequestProxy`].
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `options` - [`EmailOptions`]
    ///
    /// [`EmailOptions`]: ./struct.EmailOptions.html
    /// [`RequestProxy`]: ../../request/struct.RequestProxy.html
    #[dbus_proxy(object = "Request")]
    fn compose_email(&self, parent_window: WindowIdentifier, options: EmailOptions);

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

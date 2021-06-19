//! # Examples
//!
//! Compose an email
//!
//! ```rust,no_run
//! use ashpd::desktop::email::{EmailOptions, EmailProxy};
//! use ashpd::WindowIdentifier;
//! use std::fs::File;
//! use std::os::unix::io::AsRawFd;
//! use zvariant::Fd;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = EmailProxy::new(&connection).await?;
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!     proxy
//!         .compose_email(
//!             WindowIdentifier::default(),
//!             EmailOptions::default()
//!                 .address("test@gmail.com")
//!                 .subject("email subject")
//!                 .body("the pre-filled email body")
//!                 .attach(Fd::from(file.as_raw_fd())),
//!         )
//!         .await?;
//!
//!     Ok(())
//! }
//! ```
use zvariant::Fd;
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{
    helpers::{call_basic_response_method, property},
    Error, HandleToken, WindowIdentifier,
};

#[derive(SerializeDict, DeserializeDict, TypeDict, Clone, Debug, Default)]
/// Specified options for a [`EmailProxy::compose_email`] request.
pub struct EmailOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
    /// The email address to send to.
    address: Option<String>,
    /// The email addresses to send to.
    addresses: Option<Vec<String>>,
    /// The email addresses to CC.
    cc: Option<Vec<String>>,
    /// The email addresses to BCC.
    bcc: Option<Vec<String>>,
    /// The subject of the email.
    subject: Option<String>,
    /// The body of the email.
    body: Option<String>,
    /// A list of file descriptors of files to attach.
    attachment_fds: Option<Vec<Fd>>,
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
    pub fn addresses(mut self, addresses: &[&str]) -> Self {
        self.addresses = Some(addresses.to_vec().iter().map(|s| s.to_string()).collect());
        self
    }

    /// Sets a list of email addresses to BCC.
    pub fn bcc(mut self, bcc: &[&str]) -> Self {
        self.bcc = Some(bcc.to_vec().iter().map(|s| s.to_string()).collect());
        self
    }

    /// Sets a list of email addresses to CC.
    pub fn cc(mut self, cc: &[&str]) -> Self {
        self.cc = Some(cc.to_vec().iter().map(|s| s.to_string()).collect());
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

/// The interface lets sandboxed applications request sending an email.
pub struct EmailProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> EmailProxy<'a> {
    /// Create a new instance of [`EmailProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<EmailProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Email")
            .path("/org/freedesktop/portal/desktop")?
            .destination("org.freedesktop.portal.Desktop")
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Presents a window that lets the user compose an email.
    ///
    /// **Note** that the default email client for the host will need to support
    /// mailto: URIs following RFC 2368.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `options` - [`EmailOptions`].
    pub async fn compose_email(
        &self,
        parent_window: WindowIdentifier,
        options: EmailOptions,
    ) -> Result<(), Error> {
        call_basic_response_method(&self.0, "ComposeEmail", &(parent_window, options)).await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}

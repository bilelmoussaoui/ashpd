//! # Examples
//!
//! Compose an email
//!
//!```rust,no_run
//! use ashpd::desktop::email::{self, Email};
//! use ashpd::WindowIdentifier;
//! use std::fs::File;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!     email::compose(
//!         &WindowIdentifier::default(),
//!         Email::new()
//!             .address("test@gmail.com")
//!             .subject("email subject")
//!             .body("the pre-filled email body")
//!             .attach(&file),
//!     )
//!     .await;
//!     Ok(())
//! }
//! ```
//!
//! Or by using the Proxy directly
//!
//! ```rust,no_run
//! use ashpd::desktop::email::{Email, EmailProxy};
//! use ashpd::WindowIdentifier;
//! use std::fs::File;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = EmailProxy::new(&connection).await?;
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!     proxy
//!         .compose_email(
//!             &WindowIdentifier::default(),
//!             Email::new()
//!                 .address("test@gmail.com")
//!                 .subject("email subject")
//!                 .body("the pre-filled email body")
//!                 .attach(&file),
//!         )
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

use std::os::unix::prelude::AsRawFd;

use serde::Serialize;
use zbus::zvariant::{DeserializeDict, Fd, SerializeDict, Type};

use super::{HandleToken, DESTINATION, PATH};
use crate::{helpers::call_basic_response_method, Error, WindowIdentifier};

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`EmailProxy::compose_email`] request.
#[zvariant(signature = "dict")]
pub struct Email {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
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

impl Email {
    /// Create a new instance of [`Email`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Similar to `set_address`.
    #[must_use]
    pub fn address(mut self, address: &str) -> Self {
        self.address = Some(address.to_string());
        self
    }

    /// Sets the email address to send the email to.
    pub fn set_address(&mut self, address: &str) {
        self.address = Some(address.to_string());
    }

    /// Similar to `set_addresses`.
    #[must_use]
    pub fn addresses(mut self, addresses: &[impl AsRef<str> + Type + Serialize]) -> Self {
        self.addresses = Some(addresses.iter().map(|s| s.as_ref().to_string()).collect());
        self
    }

    /// Sets a list of email addresses to send the email to.
    pub fn set_addresses(&mut self, addresses: &[impl AsRef<str> + Type + Serialize]) {
        self.addresses = Some(addresses.iter().map(|s| s.as_ref().to_string()).collect());
    }

    /// Sets a list of email addresses to BCC.
    #[must_use]
    pub fn bcc(mut self, bcc: &[impl AsRef<str> + Type + Serialize]) -> Self {
        self.bcc = Some(bcc.iter().map(|s| s.as_ref().to_string()).collect());
        self
    }

    /// Sets a list of email addresses to BCC.
    pub fn set_bcc(&mut self, bcc: &[impl AsRef<str> + Type + Serialize]) {
        self.bcc = Some(bcc.iter().map(|s| s.as_ref().to_string()).collect());
    }

    /// Sets a list of email addresses to CC.
    #[must_use]
    pub fn cc(mut self, cc: &[impl AsRef<str> + Type + Serialize]) -> Self {
        self.cc = Some(cc.iter().map(|s| s.as_ref().to_string()).collect());
        self
    }

    /// Sets a list of email addresses to CC.
    pub fn set_cc(&mut self, cc: &[impl AsRef<str> + Type + Serialize]) {
        self.cc = Some(cc.iter().map(|s| s.as_ref().to_string()).collect());
    }

    /// Similar to `set_subject`.
    #[must_use]
    pub fn subject(mut self, subject: &str) -> Self {
        self.subject = Some(subject.to_string());
        self
    }

    /// Sets the email subject.
    pub fn set_subject(&mut self, subject: &str) {
        self.subject = Some(subject.to_string());
    }

    /// Similar to `set_body`.
    #[must_use]
    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }

    /// Sets the email body.
    pub fn set_body(&mut self, body: &str) {
        self.body = Some(body.to_string());
    }

    /// Attaches a file to the email.
    #[must_use]
    pub fn attach(mut self, attachment: &impl AsRawFd) -> Self {
        let attachment = Fd::from(attachment.as_raw_fd());
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
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Email`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Email).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Email")]
pub struct EmailProxy<'a>(zbus::Proxy<'a>);

impl<'a> EmailProxy<'a> {
    /// Create a new instance of [`EmailProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<EmailProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Email")?
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

    /// Presents a window that lets the user compose an email.
    ///
    /// **Note** the default email client for the host will need to support
    /// `mailto:` URIs following RFC 2368.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    /// * `email` - An [`Email`].
    ///
    /// # Specifications
    ///
    /// See also [`ComposeEmail`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Email.ComposeEmail).
    #[doc(alias = "ComposeEmail")]
    pub async fn compose_email(
        &self,
        identifier: &WindowIdentifier,
        email: Email,
    ) -> Result<(), Error> {
        call_basic_response_method(
            self.inner(),
            &email.handle_token,
            "ComposeEmail",
            &(&identifier, &email),
        )
        .await
    }
}

/// A handy wrapper around [`EmailProxy::compose_email`]
#[doc(alias = "xdp_portal_compose_email")]
pub async fn compose(identifier: &WindowIdentifier, email: Email) -> Result<(), Error> {
    let connection = zbus::Connection::session().await?;
    let proxy = EmailProxy::new(&connection).await?;
    proxy.compose_email(identifier, email).await?;

    Ok(())
}

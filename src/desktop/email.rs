//! Compose an email.
//!
//! Wrapper of the DBus interface: [`org.freedesktop.portal.Email`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Email).
//!
//! # Examples
//!
//! Compose an email
//!
//! ```rust,no_run
//! use std::fs::File;
//!
//! use ashpd::desktop::email::EmailRequest;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let file = File::open("/home/bilelmoussaoui/Downloads/adwaita-night.jpg").unwrap();
//!     EmailRequest::default()
//!         .address("test@gmail.com")
//!         .subject("email subject")
//!         .body("the pre-filled email body")
//!         .attach(&file)
//!         .build()
//!         .await;
//!     Ok(())
//! }
//! ```

use std::os::unix::prelude::AsRawFd;

use serde::Serialize;
use zbus::zvariant::{Fd, SerializeDict, Type};

use super::{HandleToken, DESTINATION, PATH};
use crate::{
    helpers::{call_basic_response_method, session_connection},
    Error, WindowIdentifier,
};

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct EmailOptions {
    handle_token: HandleToken,
    address: Option<String>,
    addresses: Option<Vec<String>>,
    cc: Option<Vec<String>>,
    bcc: Option<Vec<String>>,
    subject: Option<String>,
    body: Option<String>,
    attachment_fds: Option<Vec<Fd>>,
}

#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Email")]
struct EmailProxy<'a>(zbus::Proxy<'a>);

impl<'a> EmailProxy<'a> {
    /// Create a new instance of [`EmailProxy`].
    pub async fn new() -> Result<EmailProxy<'a>, Error> {
        let connection = session_connection().await?;
        let proxy = zbus::ProxyBuilder::new_bare(&connection)
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
    /// * `options` - An [`EmailOptions`].
    ///
    /// # Specifications
    ///
    /// See also [`ComposeEmail`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Email.ComposeEmail).
    #[doc(alias = "ComposeEmail")]
    pub async fn compose(
        &self,
        identifier: &WindowIdentifier,
        options: EmailOptions,
    ) -> Result<(), Error> {
        call_basic_response_method(
            self.inner(),
            &options.handle_token,
            "ComposeEmail",
            &(&identifier, &options),
        )
        .await
    }
}

#[derive(Debug, Default)]
#[doc(alias = "xdp_portal_compose_email")]
pub struct EmailRequest {
    identifier: WindowIdentifier,
    options: EmailOptions,
}

impl EmailRequest {
    /// Sets a window identifier.
    #[must_use]
    pub fn identifier(mut self, identifier: WindowIdentifier) -> Self {
        self.identifier = identifier;
        self
    }

    /// Sets the email address to send the email to.
    #[must_use]
    pub fn address(mut self, address: &str) -> Self {
        self.set_address(address);
        self
    }

    pub fn set_address(&mut self, address: &str) {
        self.options.address = Some(address.to_owned());
    }

    /// Sets a list of email addresses to send the email to.
    #[must_use]
    pub fn addresses(mut self, addresses: &[impl AsRef<str> + Type + Serialize]) -> Self {
        self.set_addresses(addresses);
        self
    }

    pub fn set_addresses(&mut self, addresses: &[impl AsRef<str> + Type + Serialize]) {
        self.options.addresses = Some(addresses.iter().map(|s| s.as_ref().to_owned()).collect());
    }

    /// Sets a list of email addresses to BCC.
    #[must_use]
    pub fn bcc(mut self, bcc: &[impl AsRef<str> + Type + Serialize]) -> Self {
        self.set_bcc(bcc);
        self
    }

    pub fn set_bcc(&mut self, bcc: &[impl AsRef<str> + Type + Serialize]) {
        self.options.bcc = Some(bcc.iter().map(|s| s.as_ref().to_owned()).collect());
    }

    /// Sets a list of email addresses to CC.
    #[must_use]
    pub fn cc(mut self, cc: &[impl AsRef<str> + Type + Serialize]) -> Self {
        self.set_cc(cc);
        self
    }

    pub fn set_cc(&mut self, cc: &[impl AsRef<str> + Type + Serialize]) {
        self.options.cc = Some(cc.iter().map(|s| s.as_ref().to_owned()).collect());
    }

    /// Sets the email subject.
    #[must_use]
    pub fn subject(mut self, subject: &str) -> Self {
        self.set_subject(subject);
        self
    }

    pub fn set_subject(&mut self, subject: &str) {
        self.options.subject = Some(subject.to_owned());
    }

    /// Sets the email body.
    #[must_use]
    pub fn body(mut self, body: &str) -> Self {
        self.set_body(body);
        self
    }

    pub fn set_body(&mut self, body: &str) {
        self.options.body = Some(body.to_owned());
    }

    /// Attaches a file to the email.
    #[must_use]
    pub fn attach(mut self, attachment: &impl AsRawFd) -> Self {
        self.add_attachment(attachment);
        self
    }

    pub fn add_attachment(&mut self, attachment: &impl AsRawFd) {
        let attachment = Fd::from(attachment.as_raw_fd());
        match self.options.attachment_fds {
            Some(ref mut attachments) => attachments.push(attachment),
            None => {
                self.options.attachment_fds.replace(vec![attachment]);
            }
        };
    }

    pub async fn build(self) -> Result<(), Error> {
        let proxy = EmailProxy::new().await?;
        proxy.compose(&self.identifier, self.options).await
    }
}

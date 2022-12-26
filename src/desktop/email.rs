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

use std::os::fd::AsFd;

use serde::Serialize;
use zbus::zvariant::{SerializeDict, Type};

use super::{HandleToken, DESTINATION, PATH};
use crate::{
    fd::Fd,
    helpers::{call_basic_response_method, session_connection},
    Error, WindowIdentifier,
};

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct EmailOptions<'f> {
    handle_token: HandleToken,
    address: Option<String>,
    addresses: Option<Vec<String>>,
    cc: Option<Vec<String>>,
    bcc: Option<Vec<String>>,
    subject: Option<String>,
    body: Option<String>,
    attachment_fds: Option<Vec<Fd<'f>>>,
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
        options: EmailOptions<'_>,
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
pub struct EmailRequest<'f> {
    identifier: WindowIdentifier,
    options: EmailOptions<'f>,
}

impl<'f> EmailRequest<'f> {
    /// Sets a window identifier.
    #[must_use]
    pub fn identifier(mut self, identifier: impl Into<Option<WindowIdentifier>>) -> Self {
        self.identifier = identifier.into().unwrap_or_default();
        self
    }

    /// Sets the email address to send the email to.
    #[must_use]
    pub fn address<'a>(mut self, address: impl Into<Option<&'a str>>) -> Self {
        self.options.address = address.into().map(ToOwned::to_owned);
        self
    }

    /// Sets a list of email addresses to send the email to.
    #[must_use]
    pub fn addresses<P: IntoIterator<Item = I>, I: AsRef<str> + Type + Serialize>(
        mut self,
        addresses: impl Into<Option<P>>,
    ) -> Self {
        self.options.addresses = addresses
            .into()
            .map(|a| a.into_iter().map(|s| s.as_ref().to_owned()).collect());
        self
    }

    /// Sets a list of email addresses to BCC.
    #[must_use]
    pub fn bcc<P: IntoIterator<Item = I>, I: AsRef<str> + Type + Serialize>(
        mut self,
        bcc: impl Into<Option<P>>,
    ) -> Self {
        self.options.bcc = bcc
            .into()
            .map(|a| a.into_iter().map(|s| s.as_ref().to_owned()).collect());
        self
    }

    /// Sets a list of email addresses to CC.
    #[must_use]
    pub fn cc<P: IntoIterator<Item = I>, I: AsRef<str> + Type + Serialize>(
        mut self,
        cc: impl Into<Option<P>>,
    ) -> Self {
        self.options.cc = cc
            .into()
            .map(|a| a.into_iter().map(|s| s.as_ref().to_owned()).collect());
        self
    }

    /// Sets the email subject.
    #[must_use]
    pub fn subject<'a>(mut self, subject: impl Into<Option<&'a str>>) -> Self {
        self.options.subject = subject.into().map(ToOwned::to_owned);
        self
    }

    /// Sets the email body.
    #[must_use]
    pub fn body<'a>(mut self, body: impl Into<Option<&'a str>>) -> Self {
        self.options.body = body.into().map(ToOwned::to_owned);
        self
    }

    /// Attaches a file to the email.
    #[must_use]
    pub fn attach(mut self, attachment: &'f impl AsFd) -> Self {
        self.add_attachment(attachment);
        self
    }

    pub fn add_attachment(&mut self, attachment: &'f impl AsFd) {
        let attachment = Fd::from(attachment);
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

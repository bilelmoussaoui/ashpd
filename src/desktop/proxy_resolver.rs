//! **Note** This portal doesn't work for sandboxed applications.
//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::proxy_resolver::ProxyResolver;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let proxy = ProxyResolver::new().await?;
//!     let url = url::Url::parse("www.google.com").unwrap();
//!
//!     println!("{:#?}", proxy.lookup(&url).await?);
//!
//!     Ok(())
//! }
//! ```

use super::{DESTINATION, PATH};
use crate::{
    helpers::{call_method, session_connection},
    Error,
};

/// The interface provides network proxy information to sandboxed applications.
///
/// It is not a portal in the strict sense, since it does not involve user
/// interaction. Applications are expected to use this interface indirectly,
/// via a library API such as the GLib [`gio::ProxyResolver`](https://gtk-rs.org/gtk-rs-core/stable/latest/docs/gio/struct.ProxyResolver.html) interface.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.ProxyResolver`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.ProxyResolver).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.ProxyResolver")]
pub struct ProxyResolver<'a>(zbus::Proxy<'a>);

impl<'a> ProxyResolver<'a> {
    /// Create a new instance of [`ProxyResolver`].
    pub async fn new() -> Result<ProxyResolver<'a>, Error> {
        let connection = session_connection().await?;
        let proxy = zbus::ProxyBuilder::new_bare(&connection)
            .interface("org.freedesktop.portal.ProxyResolver")?
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

    /// Looks up which proxy to use to connect to `uri`.
    ///
    /// # Returns
    ///
    /// A list of proxy uris of the form `protocol://[user[:password]host:port`
    /// The protocol can be `http`, `rtsp`, `socks` or another proxying
    /// protocol. `direct://` is used when no proxy is needed.
    ///
    /// # Specifications
    ///
    /// See also [`Lookup`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-ProxyResolver.Lookup).
    #[doc(alias = "Lookup")]
    pub async fn lookup(&self, uri: &url::Url) -> Result<Vec<url::Url>, Error> {
        call_method(self.inner(), "Lookup", &(uri)).await
    }
}

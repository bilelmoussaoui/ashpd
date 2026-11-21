//! # Examples
//!
//! ```rust,no_run
//! use std::{io::Read, os::fd::AsFd};
//!
//! use ashpd::desktop::secret::Secret;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let secret = Secret::new().await?;
//!
//!     let (mut x1, x2) = std::os::unix::net::UnixStream::pair()?;
//!     secret.retrieve(&x2, Default::default()).await?;
//!     drop(x2);
//!     let mut buf = Vec::new();
//!     x1.read_to_end(&mut buf)?;
//!
//!     Ok(())
//! }
//! ```

use std::{
    io::Read,
    os::{fd::AsFd, unix::net::UnixStream},
};

use serde::{Deserialize, Serialize};
use zbus::wire::{
    Fd, Type,
    as_variant::{self, optional},
};

use super::{HandleToken, Request};
use crate::{
    Error,
    activation_token::ActivationToken,
    proxy::Proxy,
    window_identifier::{WindowIdentifier, WindowIdentifierType},
};

#[derive(Serialize, Deserialize, Type, Debug, Default)]
/// Specified options for a [`Secret::retrieve`] request.
///
/// The struct uses `parent_window` and `parent_window_type` depending on
/// whether this is serialized or deserialized. i.e. on the client side
/// `parent_window` is used and `parent_window_type` will be used in the backend
/// if the `backend` feature is enabled.
#[zbus(signature = "dict")]
pub struct RetrieveOptions {
    #[serde(with = "as_variant", skip_deserializing)]
    handle_token: HandleToken,
    /// A string returned by a previous call to `retrieve`.
    /// TODO: seems to not be used by the portal...
    #[serde(default, with = "optional", skip_serializing_if = "Option::is_none")]
    token: Option<String>,
    /// A token that can be used to activate a window that might be shown in
    /// response of this request. This option was added in version 2 of the
    /// interface.
    #[serde(default, with = "optional", skip_serializing_if = "Option::is_none")]
    activation_token: Option<ActivationToken>,
    /// Identifier for the application window. This option was added in version
    /// 2 of the interface.
    #[serde(
        default,
        skip_deserializing,
        with = "optional",
        skip_serializing_if = "Option::is_none"
    )]
    parent_window: Option<WindowIdentifier>,

    /// Identifier for the application window. This option was added in version
    /// 2 of the interface.
    #[cfg(feature = "backend")]
    #[serde(default, rename(deserialize = "parent_window"), skip_serializing)]
    parent_window_type: Option<WindowIdentifierType>,
}

impl RetrieveOptions {
    #[cfg(feature = "backend")]
    /// Gets the window identifier.
    pub fn parent_window(&self) -> Option<&WindowIdentifierType> {
        self.parent_window_type.as_ref()
    }

    #[cfg(feature = "backend")]
    /// Gets the activation token.
    pub fn activation_token(&self) -> Option<&ActivationToken> {
        self.activation_token.as_ref()
    }

    #[must_use]
    /// Sets a window identifier.
    pub fn set_parent_window(mut self, parent_window: impl Into<Option<WindowIdentifier>>) -> Self {
        self.parent_window = parent_window.into();
        self
    }

    #[must_use]
    /// Sets an activation token.
    pub fn set_activation_token(
        mut self,
        activation_token: impl Into<Option<ActivationToken>>,
    ) -> Self {
        self.activation_token = activation_token.into();
        self
    }
}

/// The interface lets sandboxed applications retrieve a per-application secret.
///
/// The secret can then be used for encrypting confidential data inside the
/// sandbox.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Secret`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Secret.html).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Secret")]
pub struct Secret(Proxy<'static>);

impl Secret {
    /// Create a new instance of [`Secret`].
    pub async fn new() -> Result<Secret, Error> {
        let proxy = Proxy::new_desktop("org.freedesktop.portal.Secret").await?;
        Ok(Self(proxy))
    }

    /// Create a new instance of [`Secret`].
    pub async fn with_connection(connection: zbus::Connection) -> Result<Secret, Error> {
        let proxy =
            Proxy::new_desktop_with_connection(connection, "org.freedesktop.portal.Secret").await?;
        Ok(Self(proxy))
    }

    /// Returns the version of the portal interface.
    pub fn version(&self) -> u32 {
        self.0.version()
    }

    /// Retrieves a master secret for a sandboxed application.
    ///
    /// # Arguments
    ///
    /// * `fd` - Writaeble file descriptor for transporting the secret.
    ///
    /// # Specifications
    ///
    /// See also [`RetrieveSecret`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Secret.html#org-freedesktop-portal-secret-retrievesecret)
    #[doc(alias = "RetrieveSecret")]
    pub async fn retrieve(
        &self,
        fd: &impl AsFd,
        options: RetrieveOptions,
    ) -> Result<Request<()>, Error> {
        self.0
            .empty_request(
                &options.handle_token,
                "RetrieveSecret",
                &(Fd::from(fd), &options),
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use zbus::wire::{Dict, Endian, Signature, serialized::Context, to_bytes_for_signature};

    use super::*;

    #[test]
    fn deserialize_options_without_token() {
        // What a backend receives: the frontend strips `handle_token` and
        // apps rarely set `token` (it is only set on subsequent calls).
        let dict = Dict::new(
            &Signature::from_bytes(b"s").unwrap(),
            &Signature::from_bytes(b"v").unwrap(),
        );

        let ctxt = Context::new(Endian::Little, 0);
        let encoded = to_bytes_for_signature(ctxt, "a{sv}", &dict).unwrap();
        let (options, _) = encoded.deserialize::<RetrieveOptions>().unwrap();
        assert_eq!(options.token, None);
    }
}

impl std::ops::Deref for Secret {
    type Target = zbus::Proxy<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A handy wrapper around [`Secret::retrieve`].
///
/// It creates a UnixStream internally for receiving the secret.
pub async fn retrieve() -> Result<Vec<u8>, Error> {
    let proxy = Secret::new().await?;

    let (mut x1, x2) = UnixStream::pair()?;
    let request = proxy.retrieve(&x2, Default::default()).await?;
    drop(x2);
    // A refused request sends no secret, and the backend may keep its copy of
    // the fd open, so reading would wait for an EOF that never comes.
    request.response()?;

    // Read the secret on a blocking thread since it's a small amount of data
    #[cfg(feature = "tokio")]
    let buf = tokio::task::spawn_blocking(move || {
        let mut buf = Vec::with_capacity(64);
        x1.read_to_end(&mut buf)?;
        Ok::<_, std::io::Error>(buf)
    })
    .await
    .map_err(|e| Error::from(std::io::Error::other(e)))??;

    #[cfg(not(feature = "tokio"))]
    let buf = {
        let mut buf = Vec::with_capacity(64);
        x1.read_to_end(&mut buf)?;
        buf
    };

    Ok(buf)
}

use std::{collections::HashMap, convert::TryFrom, fmt::Debug};

use serde::{Serialize, Serializer};
use zbus::zvariant::{ObjectPath, OwnedValue, Signature, Type};

use crate::{
    desktop::{HandleToken, DESTINATION},
    helpers::{call_method, receive_signal, session_connection},
    Error,
};

pub type SessionDetails = HashMap<String, OwnedValue>;

/// Shared by all portal interfaces that involve long lived sessions.
///
/// When a method that creates a session is called, if successful, the reply
/// will include a session handle (i.e. object path) for a Session object, which
/// will stay alive for the duration of the session.
///
/// The duration of the session is defined by the interface that creates it.
/// For convenience, the interface contains a method [`Session::close`],
/// and a signal [`Session::receive_closed`]. Whether it is allowed to
/// directly call [`Session::close`] depends on the interface.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Session`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Session).
#[doc(alias = "org.freedesktop.portal.Session")]
pub struct Session<'a>(zbus::Proxy<'a>);

impl<'a> Session<'a> {
    /// Create a new instance of [`Session`].
    ///
    /// **Note** A [`Session`] is not supposed to be created manually.
    pub(crate) async fn new(path: ObjectPath<'a>) -> Result<Session<'a>, Error> {
        let connection = session_connection().await?;
        let proxy = zbus::ProxyBuilder::new_bare(&connection)
            .interface("org.freedesktop.portal.Session")?
            .path(path)?
            .destination(DESTINATION)?
            .build()
            .await?;
        Ok(Self(proxy))
    }

    pub(crate) async fn from_unique_name(
        handle_token: &HandleToken,
    ) -> Result<Session<'a>, crate::Error> {
        let connection = session_connection().await?;
        let unique_name = connection.unique_name().unwrap();
        let unique_identifier = unique_name.trim_start_matches(':').replace('.', "_");
        let path = ObjectPath::try_from(format!(
            "/org/freedesktop/portal/desktop/session/{unique_identifier}/{handle_token}",
        ))
        .unwrap();
        #[cfg(feature = "tracing")]
        tracing::info!("Creating a org.freedesktop.portal.Session {}", path);
        Self::new(path).await
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::Proxy<'_> {
        &self.0
    }

    /// Emitted when a session is closed.
    ///
    /// # Specifications
    ///
    /// See also [`Closed`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-signal-org-freedesktop-portal-Session.Closed).
    #[doc(alias = "Closed")]
    pub async fn receive_closed(&self) -> Result<SessionDetails, Error> {
        receive_signal(self.inner(), "Closed").await
    }

    /// Closes the portal session to which this object refers and ends all
    /// related user interaction (dialogs, etc).
    ///
    /// # Specifications
    ///
    /// See also [`Close`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Session.Close).
    #[doc(alias = "Close")]
    pub async fn close(&self) -> Result<(), Error> {
        call_method(self.inner(), "Close", &()).await
    }
}

impl<'a> Serialize for Session<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ObjectPath::serialize(self.inner().path(), serializer)
    }
}

impl<'a> Type for Session<'a> {
    fn signature() -> Signature<'static> {
        ObjectPath::signature()
    }
}

impl<'a> Debug for Session<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Session")
            .field(&self.inner().path().as_str())
            .finish()
    }
}

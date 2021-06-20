use crate::{
    desktop::{HandleToken, DESTINATION},
    helpers::call_method,
    Error,
};
use futures::prelude::stream::*;
use serde::{Serialize, Serializer};
use std::{collections::HashMap, convert::TryFrom};
use zvariant::{ObjectPath, OwnedValue, Signature};

pub type SessionDetails = HashMap<String, OwnedValue>;

/// The Session interface is shared by all portal interfaces that involve long
/// lived sessions. When a method that creates a session is called, if
/// successful, the reply will include a session handle (i.e. object path) for a
/// Session object, which will stay alive for the duration of the session.
///
/// The duration of the session is defined by the interface that creates it.
/// For convenience, the interface contains a method [`SessionProxy::close`], and a signal
/// [`SessionProxy::receive_closed`]. Whether it is allowed to directly
/// call [`SessionProxy::close`] depends on the interface.
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Session")]
pub struct SessionProxy<'a>(zbus::azync::Proxy<'a>, zbus::azync::Connection);

impl<'a> SessionProxy<'a> {
    /// Create a new instance of [`SessionProxy`].
    ///
    /// **Note** A [`SessionProxy`] is not supposed to be created manually.
    pub(crate) async fn new(
        connection: &zbus::azync::Connection,
        path: ObjectPath<'a>,
    ) -> Result<SessionProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Session")
            .path(path)?
            .destination(DESTINATION)
            .build_async()
            .await?;
        Ok(Self(proxy, connection.clone()))
    }

    pub(crate) async fn from_unique_name(
        connection: &zbus::azync::Connection,
        handle_token: &HandleToken,
    ) -> Result<SessionProxy<'a>, crate::Error> {
        let unique_name = connection.unique_name().unwrap();
        let unique_identifier = unique_name.trim_start_matches(':').replace('.', "_");
        let path = zvariant::ObjectPath::try_from(format!(
            "/org/freedesktop/portal/desktop/session/{}/{}",
            unique_identifier, handle_token
        ))?;
        SessionProxy::new(connection, path).await
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::azync::Proxy<'_> {
        &self.0
    }

    /// Emitted when a session is closed.
    #[doc(alias = "Closed")]
    pub async fn receive_closed(&self) -> Result<SessionDetails, Error> {
        let mut stream = self.0.receive_signal("Closed").await?;
        let message = stream.next().await.ok_or(Error::NoResponse)?;
        message.body::<SessionDetails>().map_err(From::from)
    }

    /// Closes the portal session to which this object refers and ends all
    /// related user interaction (dialogs, etc).
    #[doc(alias = "Close")]
    pub async fn close(&self) -> Result<(), Error> {
        call_method(&self.0, "Close", &()).await
    }
}

impl<'a> Serialize for SessionProxy<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        zvariant::ObjectPath::serialize(self.0.path(), serializer)
    }
}

impl<'a> zvariant::Type for SessionProxy<'a> {
    fn signature() -> Signature<'static> {
        zvariant::ObjectPath::signature()
    }
}

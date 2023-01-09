use std::{fmt::Debug, ops::Deref};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use zbus::zvariant::{ObjectPath, OwnedValue, Type};

use crate::{
    desktop::{HandleToken, Request},
    helpers::session_connection,
    Error, PortalError,
};

pub(crate) const DESKTOP_DESTINATION: &str = "org.freedesktop.portal.Desktop";
pub(crate) const DESKTOP_PATH: &str = "/org/freedesktop/portal/desktop";

pub(crate) const DOCUMENTS_DESTINATION: &str = "org.freedesktop.portal.Documents";
pub(crate) const DOCUMENTS_PATH: &str = "/org/freedesktop/portal/documents";

pub(crate) const FLATPAK_DESTINATION: &str = "org.freedesktop.portal.Flatpak";
pub(crate) const FLATPAK_PATH: &str = "/org/freedesktop/portal/Flatpak";

#[derive(Debug)]
pub struct Proxy<'a>(zbus::Proxy<'a>);

impl<'a> Proxy<'a> {
    pub async fn new<P>(
        interface: &'a str,
        path: P,
        destination: &'a str,
    ) -> Result<Proxy<'a>, Error>
    where
        P: TryInto<ObjectPath<'a>>,
        P::Error: Into<zbus::Error>,
    {
        let connection = session_connection().await?;
        let proxy = zbus::ProxyBuilder::new_bare(&connection)
            .interface(interface)?
            .path(path)?
            .destination(destination)?
            .build()
            .await?;
        Ok(Self(proxy))
    }

    pub async fn new_desktop_with_path<P>(interface: &'a str, path: P) -> Result<Proxy<'a>, Error>
    where
        P: TryInto<ObjectPath<'a>>,
        P::Error: Into<zbus::Error>,
    {
        Self::new(interface, path, DESKTOP_DESTINATION).await
    }

    pub async fn new_desktop(interface: &'a str) -> Result<Proxy<'a>, Error> {
        Self::new(interface, DESKTOP_PATH, DESKTOP_DESTINATION).await
    }

    pub async fn new_documents(interface: &'a str) -> Result<Proxy<'a>, Error> {
        Self::new(interface, DOCUMENTS_PATH, DOCUMENTS_DESTINATION).await
    }

    pub async fn new_flatpak(interface: &'a str) -> Result<Proxy<'a>, Error> {
        Self::new(interface, FLATPAK_PATH, FLATPAK_DESTINATION).await
    }

    pub async fn new_flatpak_with_path<P>(interface: &'a str, path: P) -> Result<Proxy<'a>, Error>
    where
        P: TryInto<ObjectPath<'a>>,
        P::Error: Into<zbus::Error>,
    {
        Self::new(interface, path, FLATPAK_DESTINATION).await
    }

    pub async fn request<T>(
        &self,
        handle_token: &HandleToken,
        method_name: &str,
        body: impl Serialize + Type + Debug,
    ) -> Result<Request<T>, Error>
    where
        T: for<'de> Deserialize<'de> + Type + Debug,
    {
        let mut request = Request::from_unique_name(handle_token).await?;
        futures_util::try_join!(async { request.prepare_response().await }, async {
            self.call_method(method_name, &body)
                .await
                .map_err(From::from)
        })?;
        Ok(request)
    }

    pub(crate) async fn empty_request(
        &self,
        handle_token: &HandleToken,
        method_name: &str,
        body: impl Serialize + Type + Debug,
    ) -> Result<Request<()>, Error> {
        self.request(handle_token, method_name, body).await
    }

    pub(crate) async fn call<R>(
        &self,
        method_name: &str,
        body: impl Serialize + Type + Debug,
    ) -> Result<R, Error>
    where
        R: for<'de> Deserialize<'de> + Type,
    {
        #[cfg(feature = "tracing")]
        {
            tracing::info!("Calling method {}:{}", self.interface(), method_name);
            tracing::debug!("With body {:#?}", body);
        }
        let msg = self
            .call_method(method_name, &body)
            .await
            .map_err::<PortalError, _>(From::from)?;
        let reply = msg.body::<R>()?;
        msg.take_fds();

        Ok(reply)
    }

    pub async fn property<T>(&self, property_name: &str) -> Result<T, Error>
    where
        T: TryFrom<OwnedValue>,
        zbus::Error: From<<T as TryFrom<OwnedValue>>::Error>,
    {
        self.0
            .get_property::<T>(property_name)
            .await
            .map_err(From::from)
    }

    pub(crate) async fn signal<R>(&self, signal_name: &'static str) -> Result<R, Error>
    where
        R: for<'de> Deserialize<'de> + Type + Debug,
    {
        #[cfg(feature = "tracing")]
        tracing::info!(
            "Listening to signal '{}' on '{}'",
            signal_name,
            self.interface()
        );
        let mut stream = self
            .0
            .receive_signal(signal_name)
            .await
            .map_err::<PortalError, _>(From::from)?;
        let message = stream.next().await.ok_or(Error::NoResponse)?;
        #[cfg(feature = "tracing")]
        tracing::info!(
            "Received signal '{}' on '{}'",
            signal_name,
            self.interface()
        );
        let content = message.body::<R>()?;
        #[cfg(feature = "tracing")]
        tracing::debug!("With body {:#?}", content);
        Ok(content)
    }
}

impl<'a> Deref for Proxy<'a> {
    type Target = zbus::Proxy<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

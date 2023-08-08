use std::{fmt::Debug, ops::Deref};

use futures_util::{Stream, StreamExt};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use zbus::zvariant::{ObjectPath, OwnedValue, Type};

use crate::{
    desktop::{HandleToken, Request},
    Error, PortalError,
};

pub(crate) const DESKTOP_DESTINATION: &str = "org.freedesktop.portal.Desktop";
pub(crate) const DESKTOP_PATH: &str = "/org/freedesktop/portal/desktop";

pub(crate) const DOCUMENTS_DESTINATION: &str = "org.freedesktop.portal.Documents";
pub(crate) const DOCUMENTS_PATH: &str = "/org/freedesktop/portal/documents";

pub(crate) const FLATPAK_DESTINATION: &str = "org.freedesktop.portal.Flatpak";
pub(crate) const FLATPAK_PATH: &str = "/org/freedesktop/portal/Flatpak";

static SESSION: OnceCell<zbus::Connection> = OnceCell::new();

#[derive(Debug)]
pub struct Proxy<'a>(zbus::Proxy<'a>);

impl<'a> Proxy<'a> {
    pub(crate) async fn connection() -> zbus::Result<zbus::Connection> {
        if let Some(cnx) = SESSION.get() {
            Ok(cnx.clone())
        } else {
            let cnx = zbus::Connection::session().await?;
            SESSION.set(cnx.clone()).expect("Can't reset a OnceCell");
            Ok(cnx)
        }
    }

    pub async fn unique_name(
        prefix: &str,
        handle_token: &HandleToken,
    ) -> Result<ObjectPath<'static>, Error> {
        let connection = Self::connection().await?;
        let unique_name = connection.unique_name().unwrap();
        let unique_identifier = unique_name.trim_start_matches(':').replace('.', "_");
        ObjectPath::try_from(format!("{prefix}/{unique_identifier}/{handle_token}"))
            .map_err(From::from)
    }

    pub async fn new<P>(
        interface: &'a str,
        path: P,
        destination: &'a str,
    ) -> Result<Proxy<'a>, Error>
    where
        P: TryInto<ObjectPath<'a>>,
        P::Error: Into<zbus::Error>,
    {
        let connection = Self::connection().await?;
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
        futures_util::try_join!(request.prepare_response(), async {
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

    pub(crate) async fn signalstream_with_args<I>(
        &self,
        name: &'static str,
        args: &[(u8, &str)],
    ) -> Result<impl Stream<Item = I>, Error>
    where
        I: for<'de> Deserialize<'de> + Type,
    {
        Ok(self
            .0
            .receive_signal_with_args(name, args)
            .await?
            .filter_map(|m| async move { m.body::<I>().ok() }))
    }

    pub(crate) async fn signalstream<I>(
        &self,
        name: &'static str,
    ) -> Result<impl Stream<Item = I>, Error>
    where
        I: for<'de> Deserialize<'de> + Type,
    {
        Ok(self
            .0
            .receive_signal(name)
            .await?
            .filter_map(|m| async move { m.body::<I>().ok() }))
    }
}

impl<'a> Deref for Proxy<'a> {
    type Target = zbus::Proxy<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

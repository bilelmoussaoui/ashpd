use zbus::names::WellKnownName;

pub mod access;
pub mod account;
pub mod request;
pub mod screenshot;
pub mod secret;
pub mod settings;
pub mod wallpaper;

pub struct Backend {
    cnx: zbus::Connection,
}

impl Backend {
    pub async fn new<N: TryInto<WellKnownName<'static>>>(name: N) -> Result<Self, crate::Error>
    where
        zbus::Error: From<<N as TryInto<WellKnownName<'static>>>::Error>,
    {
        let name = name.try_into().map_err(zbus::Error::from)?;
        let cnx = zbus::ConnectionBuilder::session()?
            .name(name)?
            .build()
            .await?;

        Ok(Backend { cnx })
    }

    pub async fn with_connection<N: TryInto<WellKnownName<'static>>>(
        name: N,
        cnx: zbus::Connection,
    ) -> Result<Self, crate::Error>
    where
        zbus::Error: From<<N as TryInto<WellKnownName<'static>>>::Error>,
    {
        let name = name.try_into().map_err(zbus::Error::from)?;
        cnx.request_name(name).await?;

        Ok(Backend { cnx })
    }

    pub(crate) async fn serve<T>(&self, iface: T) -> Result<(), zbus::Error>
    where
        T: zbus::Interface,
    {
        let object_server = self.cnx().object_server();
        #[cfg(feature = "tracing")]
        tracing::debug!("Serving interface: {}", T::name());
        object_server.at(crate::proxy::DESKTOP_PATH, iface).await?;
        Ok(())
    }

    fn cnx(&self) -> &zbus::Connection {
        &self.cnx
    }
}

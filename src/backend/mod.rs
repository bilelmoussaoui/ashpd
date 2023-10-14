use zbus::names::WellKnownName;

// pub mod access;
pub mod account;
pub mod file_chooser;
pub mod request;
// pub mod session;
pub mod screenshot;
pub mod settings;
pub mod wallpaper;

// We use option to be able to take() without cloning. Unwraping is safe as they
// are set in construction.
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

    pub(crate) async fn serve(&self, iface: impl zbus::Interface) -> Result<(), zbus::Error> {
        let object_server = self.cnx().object_server();
        object_server
            .at("/org/freedesktop/portal/desktop", iface)
            .await?;
        Ok(())
    }

    fn cnx(&self) -> &zbus::Connection {
        &self.cnx
    }
}

use ashpd::{
    AppID, WindowIdentifierType,
    backend::{
        Result,
        account::{AccountImpl, UserInformationOptions},
        request::RequestImpl,
    },
    desktop::{HandleToken, account::UserInformation},
};
use async_trait::async_trait;

#[derive(Default)]
pub struct Account;

#[async_trait]
impl RequestImpl for Account {
    async fn close(&self, token: HandleToken) {
        tracing::debug!("IN Close(): {token}");
    }
}

mod fdo_account {
    #[zbus::proxy(
        default_service = "org.freedesktop.Accounts",
        interface = "org.freedesktop.Accounts.User",
        gen_blocking = false
    )]
    pub trait Accounts {
        #[zbus(property, name = "IconFile")]
        fn icon_file(&self) -> zbus::Result<String>;
        #[zbus(property, name = "UserName")]
        fn user_name(&self) -> zbus::Result<String>;
        #[zbus(property, name = "RealName")]
        fn real_name(&self) -> zbus::Result<String>;
    }
}

#[async_trait]
impl AccountImpl for Account {
    async fn get_user_information(
        &self,
        _token: HandleToken,
        _app_id: Option<AppID>,
        _window_identifier: Option<WindowIdentifierType>,
        _options: UserInformationOptions,
    ) -> Result<UserInformation> {
        // Retrieve current user information by using the
        // `org.freedesktop.Accounts` interfaces.
        let cnx = zbus::Connection::system().await?;
        let uid = nix::unistd::Uid::current().as_raw();
        let path = format!("/org/freedesktop/Accounts/User{uid}");
        let proxy = fdo_account::AccountsProxy::builder(&cnx)
            .path(path)?
            .build()
            .await?;

        let uri = format!("file://{}", proxy.icon_file().await?);

        Ok(UserInformation::new(
            &proxy.user_name().await?,
            &proxy.real_name().await?,
            url::Url::parse(&uri).map_err(|e| {
                ashpd::PortalError::Failed(format!(
                    "Failed to parse user avatar uri from `{uri}` with {e}"
                ))
            })?,
        ))
    }
}

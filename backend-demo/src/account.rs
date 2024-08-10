use ashpd::{
    backend::{
        account::{AccountImpl, UserInformationOptions},
        request::RequestImpl,
    },
    desktop::{account::UserInformation, Response},
    AppID, WindowIdentifierType,
};
use async_trait::async_trait;

#[derive(Default)]
pub struct Account;

#[async_trait]
impl RequestImpl for Account {
    async fn close(&self) {
        tracing::debug!("IN Close()");
    }
}

#[async_trait]
impl AccountImpl for Account {
    async fn get_information(
        &self,
        _app_id: AppID,
        _window_identifier: Option<WindowIdentifierType>,
        _options: UserInformationOptions,
    ) -> Response<UserInformation> {
        match UserInformation::current_user().await {
            Ok(info) => Response::ok(info),
            Err(err) => {
                tracing::error!("Failed to get user info: {err}");
                Response::other()
            }
        }
    }
}

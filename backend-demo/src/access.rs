use ashpd::{
    AppID, WindowIdentifierType,
    backend::{
        Result,
        access::{AccessImpl, AccessOptions, AccessResponse},
        request::RequestImpl,
    },
    desktop::HandleToken,
};
use async_trait::async_trait;

#[derive(Default)]
pub struct Access;

#[async_trait]
impl RequestImpl for Access {
    async fn close(&self, token: HandleToken) {
        tracing::debug!("IN Close(): {token}");
    }
}

#[async_trait]
impl AccessImpl for Access {
    async fn access_dialog(
        &self,
        _handle: HandleToken,
        _app_id: Option<AppID>,
        _window_identifier: Option<WindowIdentifierType>,
        _title: String,
        _subtitle: String,
        _body: String,
        options: AccessOptions,
    ) -> Result<AccessResponse> {
        let mut response = AccessResponse::default();
        for choice in options.choices() {
            response = response.choice(choice.id(), choice.initial_selection());
        }
        Ok(response)
    }
}

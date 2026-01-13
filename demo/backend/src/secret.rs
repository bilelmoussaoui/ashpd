use std::collections::HashMap;

use ashpd::{
    backend::{Result, request::RequestImpl, secret::SecretImpl},
    desktop::HandleToken,
    zbus::zvariant::OwnedValue,
};
use ashpd_handles::AppID;
use async_trait::async_trait;

#[derive(Default)]
pub struct Secret;

#[async_trait]
impl RequestImpl for Secret {
    async fn close(&self, token: HandleToken) {
        tracing::debug!("IN Close(): {token}");
    }
}

#[async_trait]
impl SecretImpl for Secret {
    async fn retrieve(
        &self,
        _token: HandleToken,
        _app_id: AppID,
        _fd: std::os::fd::OwnedFd,
    ) -> Result<HashMap<String, OwnedValue>> {
        Ok(Default::default())
    }
}

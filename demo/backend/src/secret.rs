use std::collections::HashMap;

use ashpd::{
    MaybeAppID,
    backend::{Result, request::RequestImpl, secret::SecretImpl},
    desktop::{HandleToken, secret::RetrieveOptions},
    zbus::wire::OwnedVariant,
};
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
        _app_id: MaybeAppID,
        _fd: std::os::fd::OwnedFd,
        _options: RetrieveOptions,
    ) -> Result<HashMap<String, OwnedVariant>> {
        Ok(Default::default())
    }
}

use std::collections::HashMap;

use ashpd::{
    backend::{request::RequestImpl, secret::SecretImpl},
    desktop::Response,
    zbus::zvariant::OwnedValue,
    AppID,
};
use async_trait::async_trait;

#[derive(Default)]
pub struct Secret;

#[async_trait]
impl RequestImpl for Secret {
    async fn close(&self) {
        tracing::debug!("IN Close()");
    }
}

#[async_trait]
impl SecretImpl for Secret {
    async fn retrieve(
        &self,
        _app_id: AppID,
        _fd: std::os::fd::OwnedFd,
    ) -> Response<HashMap<String, OwnedValue>> {
        Response::Ok(Default::default())
    }
}

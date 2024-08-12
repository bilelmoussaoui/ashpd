use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use zbus::zvariant::{self, OwnedValue};

use crate::{
    backend::request::{Request, RequestImpl},
    desktop::Response,
    AppID,
};

#[async_trait]
pub trait SecretImpl: RequestImpl {
    async fn retrieve(
        &self,
        app_id: AppID,
        fd: zvariant::OwnedFd,
    ) -> Response<HashMap<String, OwnedValue>>;
}

pub struct SecretInterface {
    imp: Arc<dyn SecretImpl>,
    cnx: zbus::Connection,
}

impl SecretInterface {
    pub fn new(imp: impl SecretImpl + 'static, cnx: zbus::Connection) -> Self {
        Self {
            imp: Arc::new(imp),
            cnx,
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Secret")]
impl SecretInterface {
    #[dbus_interface(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        1
    }

    #[dbus_interface(out_args("response", "results"))]
    async fn retrieve_secret(
        &self,
        handle: zvariant::OwnedObjectPath,
        app_id: AppID,
        fd: zvariant::OwnedFd,
        _options: HashMap<String, OwnedValue>,
    ) -> Response<HashMap<String, OwnedValue>> {
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Secret::RetrieveSecret",
            &self.cnx,
            handle,
            Arc::clone(&self.imp),
            async move { imp.retrieve(app_id, fd).await },
        )
        .await
        .unwrap_or(Response::other())
    }
}

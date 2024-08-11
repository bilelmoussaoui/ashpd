use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use futures_util::future::abortable;
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
    imp: Arc<Box<dyn SecretImpl>>,
    cnx: zbus::Connection,
}

impl SecretInterface {
    pub fn new(imp: impl SecretImpl + 'static, cnx: zbus::Connection) -> Self {
        Self {
            imp: Arc::new(Box::new(imp)),
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
        #[zbus(object_server)] server: &zbus::object_server::ObjectServer,
        handle: zvariant::OwnedObjectPath,
        app_id: AppID,
        fd: zvariant::OwnedFd,
        _options: HashMap<String, OwnedValue>,
    ) -> Response<HashMap<String, OwnedValue>> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Secret::RetrieveSecret");

        let imp: Arc<Box<dyn SecretImpl>> = Arc::clone(&self.imp);
        let (fut, request_handle) = abortable(async { imp.retrieve(app_id, fd).await });

        let imp_request = Arc::clone(&self.imp);
        let close_cb = || {
            tokio::spawn(async move {
                RequestImpl::close(&**imp_request).await;
            });
        };
        let request = Request::new(close_cb, handle.clone(), request_handle, self.cnx.clone());
        server.at(&handle, request).await.unwrap();

        let response = fut.await.unwrap_or(Response::cancelled());
        #[cfg(feature = "tracing")]
        tracing::debug!("Releasing request {:?}", handle.as_str());
        server.remove::<Request, _>(&handle).await.unwrap();

        #[cfg(feature = "tracing")]
        tracing::debug!("Secret::RetrieveSecret returned {:#?}", response);
        response
    }
}

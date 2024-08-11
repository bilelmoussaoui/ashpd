use std::boxed::Box;

use async_trait::async_trait;
use futures_util::future::AbortHandle;
use tokio::sync::Mutex;
use zbus::zvariant::OwnedObjectPath;

#[async_trait]
pub trait RequestImpl: Send + Sync {
    async fn close(&self);
}

pub(crate) struct Request {
    close_cb: Mutex<Option<Box<dyn FnOnce() + Send + Sync>>>,
    handle_path: OwnedObjectPath,
    request_handle: AbortHandle,
    #[allow(dead_code)]
    cnx: zbus::Connection,
}

impl Request {
    pub(crate) fn new(
        close_cb: impl FnOnce() + Send + Sync + 'static,
        handle_path: OwnedObjectPath,
        request_handle: AbortHandle,
        cnx: zbus::Connection,
    ) -> Self {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            "Serving `org.freedesktop.impl.portal.Request` at {:?}",
            handle_path.as_str()
        );
        Self {
            close_cb: Mutex::new(Some(Box::new(close_cb))),
            handle_path,
            request_handle,
            cnx,
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Request")]
impl Request {
    async fn close(
        &self,
        #[zbus(object_server)] server: &zbus::ObjectServer,
    ) -> zbus::fdo::Result<()> {
        self.request_handle.abort();
        if let Some(close_cb) = self.close_cb.lock().await.take() {
            close_cb();
        }

        // Drop the request as it served it purpose once closed
        #[cfg(feature = "tracing")]
        tracing::debug!("Releasing request {:?}", self.handle_path.as_str());
        server.remove::<Self, _>(&self.handle_path).await?;
        Ok(())
    }
}

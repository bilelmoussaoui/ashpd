use std::{boxed::Box, future::Future, sync::Arc};

use async_trait::async_trait;
use futures_util::future::{abortable, AbortHandle};
use tokio::sync::Mutex;
use zbus::zvariant::{ObjectPath, OwnedObjectPath};

use crate::desktop::{HandleToken, Response};

#[async_trait]
pub trait RequestImpl: Send + Sync {
    async fn close(&self, token: HandleToken);
}

pub struct Request {
    close_cb: Mutex<Option<Box<dyn FnOnce() + Send + Sync>>>,
    path: OwnedObjectPath,
    abort_handle: AbortHandle,
    #[allow(dead_code)]
    cnx: zbus::Connection,
}

impl Request {
    pub fn path(&self) -> ObjectPath<'_> {
        self.path.as_ref()
    }

    pub(crate) async fn spawn<T, R>(
        _method: &'static str,
        cnx: &zbus::Connection,
        path: OwnedObjectPath,
        imp: Arc<R>,
        callback: impl Future<Output = crate::backend::Result<T>>,
    ) -> crate::backend::Result<Response<T>>
    where
        R: RequestImpl + 'static + ?Sized,
        T: std::fmt::Debug,
    {
        #[cfg(feature = "tracing")]
        tracing::debug!("{_method}");
        let (fut, abort_handle) = abortable(callback);
        let token = HandleToken::try_from(&path).unwrap();
        let close_cb = || {
            tokio::spawn(async move {
                RequestImpl::close(&*imp, token).await;
            });
        };
        let request = Request::new(close_cb, path.clone(), abort_handle, cnx.clone());
        let server = cnx.object_server();
        #[cfg(feature = "tracing")]
        tracing::debug!(
            "Serving `org.freedesktop.impl.portal.Request` at {:?}",
            path.as_str()
        );
        server.at(&path, request).await?;

        let response = match fut.await {
            Err(_) => Response::cancelled(),
            Ok(response) => Response::ok(response?),
        };
        #[cfg(feature = "tracing")]
        tracing::debug!("{_method} returned {:#?}", response);
        #[cfg(feature = "tracing")]
        tracing::debug!("Releasing request {:?}", path.as_str());
        server.remove::<Self, _>(&path).await?;
        Ok(response)
    }

    pub(crate) fn new(
        close_cb: impl FnOnce() + Send + Sync + 'static,
        path: OwnedObjectPath,
        abort_handle: AbortHandle,
        cnx: zbus::Connection,
    ) -> Self {
        Self {
            close_cb: Mutex::new(Some(Box::new(close_cb))),
            path,
            abort_handle,
            cnx,
        }
    }
}

impl PartialEq for Request {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
impl Eq for Request {}

#[zbus::interface(name = "org.freedesktop.impl.portal.Request")]
impl Request {
    async fn close(
        &self,
        #[zbus(object_server)] server: &zbus::ObjectServer,
    ) -> zbus::fdo::Result<()> {
        self.abort_handle.abort();
        if let Some(close_cb) = self.close_cb.lock().await.take() {
            close_cb();
        }

        // Drop the request as it served it purpose once closed
        #[cfg(feature = "tracing")]
        tracing::debug!("Releasing request {:?}", self.path.as_str());
        server.remove::<Self, _>(&self.path).await?;
        Ok(())
    }
}

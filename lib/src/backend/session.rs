use std::{boxed::Box, future::Future, sync::Arc};

use async_trait::async_trait;
use futures_util::future::{abortable, AbortHandle};
use tokio::sync::Mutex;
use zbus::{
    object_server::SignalEmitter,
    zvariant::{ObjectPath, OwnedObjectPath, SerializeDict, Type},
};

use crate::desktop::HandleToken;

#[derive(Debug, SerializeDict, Type)]
#[zvariant(signature = "dict")]
pub struct CreateSessionResponse {
    session_id: String,
}

impl Default for CreateSessionResponse {
    fn default() -> Self {
        Self {
            session_id: HandleToken::default(),
        }
    }
}

#[async_trait]
pub trait SessionSignalEmitter: Send + Sync {
    async fn emit_closed(&self) -> zbus::Result<()>;
}

#[async_trait]
pub trait SessionImpl: Send + Sync {
    async fn close(&self);
    // Set the signal emitter, allowing to notify of changes.
    fn set_signal_emitter(&mut self, signal_emitter: Arc<dyn SessionSignalEmitter>);
}

#[derive(Clone)]
pub struct Session {
    close_cb: Mutex<Option<Box<dyn FnOnce() + Send + Sync>>>,
    path: OwnedObjectPath,
    #[allow(dead_code)]
    cnx: zbus::Connection,
}

impl Session {
    pub fn path(&self) -> ObjectPath<'_> {
        self.path.as_ref()
    }

    pub(crate) async fn spawn<R>(
        _method: &'static str,
        cnx: &zbus::Connection,
        path: OwnedObjectPath,
        imp: Arc<R>,
    ) -> crate::backend::Result<CreateSessionResponse>
    where
        R: SessionImpl + 'static + ?Sized,
    {
        #[cfg(feature = "tracing")]
        tracing::debug!("{_method}");
        let close_cb = || {
            tokio::spawn(async move {
                SessionImpl::close(&*imp).await;
            });
        };
        let session = Self::new(close_cb, path.clone(), cnx.clone());
        let server = cnx.object_server();
        #[cfg(feature = "tracing")]
        tracing::debug!(
            "Serving `org.freedesktop.impl.portal.Session` at {:?}",
            path.as_str()
        );
        server.at(&path, session).await?;

        Ok(())
    }

    pub(crate) fn new(
        close_cb: impl FnOnce() + Send + Sync + 'static,
        path: OwnedObjectPath,
        cnx: zbus::Connection,
    ) -> Self {
        Self {
            close_cb: Mutex::new(Some(Box::new(close_cb))),
            path,
            cnx,
        }
    }
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
impl Eq for Session {}

#[zbus::interface(name = "org.freedesktop.impl.portal.Session")]
impl Session {
    async fn close(
        &self,
        #[zbus(object_server)] server: &zbus::ObjectServer,
    ) -> zbus::fdo::Result<()> {
        if let Some(close_cb) = self.close_cb.lock().await.take() {
            close_cb();
        }

        // Drop the session as it served it purpose once closed
        #[cfg(feature = "tracing")]
        tracing::debug!("Releasing session {:?}", self.path.as_str());
        server.remove::<Self, _>(&self.path).await?;
        Ok(())
    }

    #[zbus(signal)]
    async fn closed(signal_ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;
}

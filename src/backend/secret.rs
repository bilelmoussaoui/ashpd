use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use futures_channel::{
    mpsc::{UnboundedReceiver as Receiver, UnboundedSender as Sender},
    oneshot,
};
use futures_util::{
    future::{try_select, Either},
    pin_mut, SinkExt, StreamExt,
};
use tokio::sync::Mutex;
use zbus::zvariant::{self, OwnedObjectPath, OwnedValue};

use crate::{
    backend::{
        request::{Request, RequestImpl},
        Backend,
    },
    desktop::Response,
    AppID,
};

#[async_trait]
pub trait SecretImpl {
    async fn retrieve(
        &self,
        app_id: AppID,
        fd: zvariant::OwnedFd,
    ) -> Response<HashMap<String, OwnedValue>>;
}

enum Action {
    RetrieveSecret(
        OwnedObjectPath,
        AppID,
        zvariant::OwnedFd,
        HashMap<String, zvariant::OwnedValue>,
        oneshot::Sender<Response<HashMap<String, OwnedValue>>>,
    ),
}

pub struct Secret<T: SecretImpl + RequestImpl> {
    receiver: Arc<Mutex<Receiver<Action>>>,
    imp: Arc<T>,
    cnx: zbus::Connection,
}

impl<T: SecretImpl + RequestImpl> Secret<T> {
    pub async fn new(imp: T, backend: &Backend) -> zbus::Result<Self> {
        let (sender, receiver) = futures_channel::mpsc::unbounded();
        let iface = SecretInterface::new(sender);
        backend.serve(iface).await?;
        let provider = Self {
            receiver: Arc::new(Mutex::new(receiver)),
            imp: Arc::new(imp),
            cnx: backend.cnx().clone(),
        };

        Ok(provider)
    }

    async fn activate(&self, action: Action) -> Result<(), crate::Error> {
        let Action::RetrieveSecret(handle_path, app_id, fd, _options, sender) = action;
        let request = Request::new(Arc::clone(&self.imp), handle_path, &self.cnx).await?;
        let imp = Arc::clone(&self.imp);
        let future1 = async {
            let result = imp.retrieve(app_id, fd).await;
            let _ = sender.send(result);
            Ok(()) as Result<(), crate::Error>
        };
        let future2 = async {
            request.next().await?;
            Ok(()) as Result<(), crate::Error>
        };

        pin_mut!(future1); // 'select' requires Future + Unpin bounds
        pin_mut!(future2);
        match try_select(future1, future2).await {
            Ok(_) => Ok(()),
            Err(Either::Left((err, _))) => Err(err),
            Err(Either::Right((err, _))) => Err(err),
        }?;
        Ok(())
    }

    pub async fn try_next(&self) -> Result<(), crate::Error> {
        if let Some(action) = (*self.receiver.lock().await).next().await {
            self.activate(action).await?;
        }
        Ok(())
    }
}

struct SecretInterface {
    sender: Arc<Mutex<Sender<Action>>>,
}

impl SecretInterface {
    pub fn new(sender: Sender<Action>) -> Self {
        Self {
            sender: Arc::new(Mutex::new(sender)),
        }
    }
}
#[zbus::interface(name = "org.freedesktop.impl.portal.Secret")]
impl SecretInterface {
    #[dbus_interface(property, name = "version")]
    fn version(&self) -> u32 {
        1
    }

    #[dbus_interface(out_args("response", "results"))]
    async fn retrieve_secret(
        &self,
        handle: zvariant::OwnedObjectPath,
        app_id: AppID,
        fd: zvariant::OwnedFd,
        options: HashMap<String, OwnedValue>,
    ) -> Response<HashMap<String, OwnedValue>> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Secret::RetrieveSecret");

        let (sender, receiver) = futures_channel::oneshot::channel();
        let _ = self
            .sender
            .lock()
            .await
            .send(Action::RetrieveSecret(handle, app_id, fd, options, sender))
            .await;
        let response = receiver.await.unwrap_or(Response::cancelled());
        #[cfg(feature = "tracing")]
        tracing::debug!("Secret::RetrieveSecret returned {:#?}", response);
        response
    }
}

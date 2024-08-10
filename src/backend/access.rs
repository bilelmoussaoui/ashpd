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

use crate::{
    backend::{
        request::{Request, RequestImpl},
        Backend,
    },
    desktop::request::Response,
    zvariant::{OwnedObjectPath, OwnedValue},
    AppID, WindowIdentifierType,
};

#[async_trait]
pub trait AccessImpl {
    async fn access_dialog(
        &self,
        app_id: AppID,
        window_identifier: Option<WindowIdentifierType>,
        title: String,
        subtitle: String,
        body: String,
        options: HashMap<String, OwnedValue>,
    ) -> Response<HashMap<String, OwnedValue>>;
}

pub struct Access<T: AccessImpl + RequestImpl> {
    receiver: Arc<Mutex<Receiver<Action>>>,
    cnx: zbus::Connection,
    imp: Arc<T>,
}

impl<T: AccessImpl + RequestImpl> Access<T> {
    pub async fn new(imp: T, backend: &Backend) -> zbus::Result<Self> {
        let (sender, receiver) = futures_channel::mpsc::unbounded();
        let iface = AccessInterface::new(sender);
        backend.serve(iface).await?;
        let provider = Self {
            receiver: Arc::new(Mutex::new(receiver)),
            imp: Arc::new(imp),
            cnx: backend.cnx().clone(),
        };

        Ok(provider)
    }

    pub async fn try_next(&self) -> Result<(), crate::Error> {
        if let Some(action) = (*self.receiver.lock().await).next().await {
            self.activate(action).await?;
        }
        Ok(())
    }

    async fn activate(&self, action: Action) -> Result<(), crate::Error> {
        let Action::AccessDialog(
            handle_path,
            app_id,
            window_identifier,
            title,
            subtitle,
            body,
            options,
            sender,
        ) = action;
        let request = Request::new(Arc::clone(&self.imp), handle_path, &self.cnx).await?;
        let imp = Arc::clone(&self.imp);
        let future1 = async {
            let result = imp
                .access_dialog(app_id, window_identifier, title, subtitle, body, options)
                .await;
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
}

enum Action {
    AccessDialog(
        OwnedObjectPath,
        AppID,
        Option<WindowIdentifierType>,
        String,
        String,
        String,
        HashMap<String, OwnedValue>,
        oneshot::Sender<Response<HashMap<String, OwnedValue>>>,
    ),
}

struct AccessInterface {
    sender: Arc<Mutex<Sender<Action>>>,
}

impl AccessInterface {
    pub fn new(sender: Sender<Action>) -> Self {
        Self {
            sender: Arc::new(Mutex::new(sender)),
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Access")]
impl AccessInterface {
    #[dbus_interface(property, name = "version")]
    fn version(&self) -> u32 {
        1 // TODO: Is this correct?
    }

    #[allow(clippy::too_many_arguments)]
    async fn access_dialog(
        &self,
        handle: OwnedObjectPath,
        app_id: AppID,
        window_identifier: &str,
        title: String,
        subtitle: String,
        body: String,
        options: HashMap<String, OwnedValue>,
    ) -> Response<HashMap<String, OwnedValue>> {
        let (sender, receiver) = futures_channel::oneshot::channel();
        #[cfg(feature = "tracing")]
        tracing::debug!("Access::AccessDialog");

        let window_identifier = if window_identifier.is_empty() {
            None
        } else {
            window_identifier.parse::<WindowIdentifierType>().ok()
        };

        let _ = self
            .sender
            .lock()
            .await
            .send(Action::AccessDialog(
                handle,
                app_id,
                window_identifier,
                title,
                subtitle,
                body,
                options,
                sender,
            ))
            .await;

        let response = receiver.await.unwrap_or(Response::cancelled());
        #[cfg(feature = "tracing")]
        tracing::debug!("Access::AccessDialog returned {:#?}", response);
        response
    }
}

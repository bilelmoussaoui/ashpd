use std::sync::Arc;

use async_lock::Mutex;
use async_trait::async_trait;
use futures_channel::{
    mpsc::{UnboundedReceiver as Receiver, UnboundedSender as Sender},
    oneshot,
};
use futures_util::{
    future::{try_select, Either},
    pin_mut, SinkExt, StreamExt,
};

use crate::{
    backend::{
        request::{Request, RequestImpl},
        Backend,
    },
    desktop::{account::UserInformation, request::Response},
    zvariant::{DeserializeDict, OwnedObjectPath, Type},
    AppID, WindowIdentifierType,
};

#[derive(Debug, DeserializeDict, Type)]
#[zvariant(signature = "dict")]
pub struct UserInformationOptions {
    reason: String,
}

impl UserInformationOptions {
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

#[async_trait]
pub trait AccountImpl: RequestImpl {
    async fn get_user_information(
        &self,
        app_id: AppID,
        window_identifier: WindowIdentifierType,
        options: UserInformationOptions,
    ) -> Response<UserInformation>;
}

pub struct Account<T: AccountImpl + RequestImpl> {
    receiver: Arc<Mutex<Receiver<Action>>>,
    cnx: zbus::Connection,
    imp: Arc<T>,
}

impl<T: AccountImpl + RequestImpl> Account<T> {
    pub async fn new(imp: T, backend: &Backend) -> zbus::Result<Self> {
        let (sender, receiver) = futures_channel::mpsc::unbounded();
        let iface = AccountInterface::new(sender);
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
        #[cfg(feature = "tracing")]
        tracing::debug!("[backend/account]: Activating action");
        let Action::GetUserInformation(handle_path, app_id, window_identifier, options, sender) =
            action;
        let request = Request::new(Arc::clone(&self.imp), handle_path, &self.cnx).await?;
        let imp = Arc::clone(&self.imp);
        let future1 = async {
            let result = imp
                .get_user_information(app_id, window_identifier, options)
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
        #[cfg(feature = "tracing")]
        tracing::debug!("[backend/account]: Action done");
        Ok(())
    }
}

enum Action {
    GetUserInformation(
        OwnedObjectPath,
        AppID,
        WindowIdentifierType,
        UserInformationOptions,
        oneshot::Sender<Response<UserInformation>>,
    ),
}

struct AccountInterface {
    sender: Arc<Mutex<Sender<Action>>>,
}

impl AccountInterface {
    pub fn new(sender: Sender<Action>) -> Self {
        Self {
            sender: Arc::new(Mutex::new(sender)),
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Account")]
impl AccountInterface {
    #[zbus(name = "GetUserInformation")]
    async fn get_user_information(
        &self,
        handle: OwnedObjectPath,
        app_id: AppID,
        window_identifier: WindowIdentifierType,
        options: UserInformationOptions,
    ) -> Response<UserInformation> {
        let (sender, receiver) = futures_channel::oneshot::channel();
        let _ = self
            .sender
            .lock()
            .await
            .send(Action::GetUserInformation(
                handle,
                app_id,
                window_identifier,
                options,
                sender,
            ))
            .await;

        receiver.await.unwrap()
    }
}

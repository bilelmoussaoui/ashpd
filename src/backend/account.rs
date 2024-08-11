use std::sync::Arc;

use async_trait::async_trait;
use futures_util::future::abortable;

use crate::{
    backend::request::{Request, RequestImpl},
    desktop::{account::UserInformation, request::Response},
    zvariant::{DeserializeDict, OwnedObjectPath, Type},
    AppID, WindowIdentifierType,
};

#[derive(Debug, DeserializeDict, Type)]
#[zvariant(signature = "dict")]
pub struct UserInformationOptions {
    reason: Option<String>,
}

impl UserInformationOptions {
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
}

#[async_trait]
pub trait AccountImpl: RequestImpl {
    async fn get_user_information(
        &self,
        app_id: Option<AppID>,
        window_identifier: Option<WindowIdentifierType>,
        options: UserInformationOptions,
    ) -> Response<UserInformation>;
}

pub struct AccountInterface {
    imp: Arc<Box<dyn AccountImpl>>,
    cnx: zbus::Connection,
}

impl AccountInterface {
    pub fn new(imp: impl AccountImpl + 'static, cnx: zbus::Connection) -> Self {
        Self {
            imp: Arc::new(Box::new(imp)),
            cnx,
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Account")]
impl AccountInterface {
    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        1
    }

    #[zbus(name = "GetUserInformation")]
    async fn get_user_information(
        &self,
        #[zbus(object_server)] server: &zbus::object_server::ObjectServer,
        handle: OwnedObjectPath,
        app_id: &str,
        window_identifier: &str,
        options: UserInformationOptions,
    ) -> Response<UserInformation> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Account::GetUserInformation");
        let window_identifier = WindowIdentifierType::from_maybe_str(window_identifier);
        let app_id = AppID::from_maybe_str(app_id);

        let imp: Arc<Box<dyn AccountImpl>> = Arc::clone(&self.imp);
        let (fut, request_handle) = abortable(async {
            imp.get_user_information(app_id, window_identifier, options)
                .await
        });

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
        tracing::debug!("Account::GetUserInformation returned {:#?}", response);
        response
    }
}

use std::sync::Arc;

use async_trait::async_trait;

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
    imp: Arc<dyn AccountImpl>,
    cnx: zbus::Connection,
}

impl AccountInterface {
    pub fn new(imp: impl AccountImpl + 'static, cnx: zbus::Connection) -> Self {
        Self {
            imp: Arc::new(imp),
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
    #[dbus_interface(out_args("response", "results"))]
    async fn get_user_information(
        &self,
        handle: OwnedObjectPath,
        app_id: &str,
        window_identifier: &str,
        options: UserInformationOptions,
    ) -> Response<UserInformation> {
        let window_identifier = WindowIdentifierType::from_maybe_str(window_identifier);
        let app_id = AppID::from_maybe_str(app_id);
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Account::GetUserInformation",
            &self.cnx,
            handle,
            Arc::clone(&self.imp),
            async move {
                imp.get_user_information(app_id, window_identifier, options)
                    .await
            },
        )
        .await
        .unwrap_or(Response::other())
    }
}

use zbus::dbus_interface;
use zvariant::DeserializeDict;

use crate::{
    desktop::{
        account::UserInfo,
        request::{Response, ResponseError},
        HandleToken,
    },
    WindowIdentifierType,
};

#[derive(Debug, DeserializeDict, zvariant::Type)]
#[zvariant(signature = "dict")]
pub struct UserInfoOptions {
    reason: Option<String>,
}

pub struct Account {}

#[dbus_interface(name = "org.freedesktop.impl.portal.Account")]
impl Account {
    async fn get_user_information(
        &self,
        handle: HandleToken,
        app_id: &str,
        window_identifier: WindowIdentifierType,
        options: UserInfoOptions,
    ) -> Response<UserInfo> {
        Response::Err(ResponseError::Cancelled)
    }
}

use std::collections::HashMap;

use zvariant::{OwnedValue, Value};

use crate::{
    desktop::{request::Response, HandleToken, ResponseError},
    WindowIdentifierType,
};

pub struct Access {}

#[zbus::nterface(name = "org.freedesktop.impl.portal.Access")]
impl Access {
    fn access_dialog(
        &self,
        _handle: HandleToken,
        _app_id: &str,
        _window_identifier: WindowIdentifierType,
        _title: &str,
        _subtitle: &str,
        _body: &str,
        _options: HashMap<&str, Value<'_>>,
    ) -> Response<HashMap<String, OwnedValue>> {
        Response::Err(ResponseError::Cancelled)
    }
}

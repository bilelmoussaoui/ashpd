use std::convert::TryFrom;

use crate::request::{BasicResponse, RequestProxy};
use crate::Error;
use zvariant::OwnedValue;

pub(crate) async fn call_request_method<R>(
    proxy: &zbus::azync::Proxy<'_>,
    method_name: &str,
    body: &(impl serde::ser::Serialize + zvariant::Type),
) -> Result<R, Error>
where
    R: serde::de::DeserializeOwned + zvariant::Type,
{
    let message = proxy.call_method(method_name, body).await?;
    let path: zvariant::OwnedObjectPath = message.body()?;
    let request = RequestProxy::new(proxy.connection(), path.into_inner()).await?;
    let response = request.receive_response::<R>().await?;
    Ok(response)
}

pub(crate) async fn call_basic_response_method(
    proxy: &zbus::azync::Proxy<'_>,
    method_name: &str,
    body: &(impl serde::ser::Serialize + zvariant::Type),
) -> Result<(), Error> {
    let message = proxy.call_method(method_name, body).await?;
    let path: zvariant::OwnedObjectPath = message.body()?;

    let request = RequestProxy::new(proxy.connection(), path.into_inner()).await?;
    request.receive_response::<BasicResponse>().await?;
    Ok(())
}

pub(crate) async fn call_method<R>(
    proxy: &zbus::azync::Proxy<'_>,
    method_name: &str,
    body: &(impl serde::ser::Serialize + zvariant::Type),
) -> Result<R, Error>
where
    R: serde::de::DeserializeOwned + zvariant::Type,
{
    proxy
        .call_method(method_name, body)
        .await?
        .body::<R>()
        .map_err(From::from)
}

pub(crate) async fn property<R>(
    proxy: &zbus::azync::Proxy<'_>,
    property_name: &str,
) -> Result<R, Error>
where
    R: serde::de::DeserializeOwned + zvariant::Type + TryFrom<OwnedValue>,
{
    proxy
        .get_property::<R>(property_name)
        .await
        .map_err(From::from)
}

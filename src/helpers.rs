use std::{convert::TryFrom, fmt::Debug};

use crate::request::{BasicResponse, RequestProxy};
use crate::Error;
use zvariant::OwnedValue;

pub(crate) async fn call_request_method<R, B>(
    proxy: &zbus::azync::Proxy<'_>,
    method_name: &str,
    body: &B,
) -> Result<R, Error>
where
    R: serde::de::DeserializeOwned + zvariant::Type + Debug,
    B: serde::ser::Serialize + zvariant::Type + Debug,
{
    println!(
        "calling request method {} with body {:#?}",
        method_name, body
    );
    let path = proxy
        .call::<B, zvariant::OwnedObjectPath>(method_name, body)
        .await?;
    println!("received path {:#?}", path);
    let request = RequestProxy::new(proxy.connection(), path.into_inner()).await?;
    println!("created request {:#?}", request);
    let response = request.receive_response::<R>().await?;
    println!("received response {:#?}", response);
    Ok(response)
}

pub(crate) async fn call_basic_response_method<B>(
    proxy: &zbus::azync::Proxy<'_>,
    method_name: &str,
    body: &B,
) -> Result<(), Error>
where
    B: serde::ser::Serialize + zvariant::Type + Debug,
{
    println!(
        "calling casic response method {} with body {:#?}",
        method_name, body
    );
    let path = proxy
        .call::<B, zvariant::OwnedObjectPath>(method_name, body)
        .await?;

    let request = RequestProxy::new(proxy.connection(), path.into_inner()).await?;
    let response = request.receive_response::<BasicResponse>().await?;
    Ok(())
}

pub(crate) async fn call_method<R, B>(
    proxy: &zbus::azync::Proxy<'_>,
    method_name: &str,
    body: &B,
) -> Result<R, Error>
where
    R: serde::de::DeserializeOwned + zvariant::Type,
    B: serde::ser::Serialize + zvariant::Type + Debug,
{
    println!("calling method {} with body {:#?}", method_name, body);
    proxy
        .call::<B, R>(method_name, body)
        .await
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

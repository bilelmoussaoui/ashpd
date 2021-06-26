use crate::desktop::{
    request::{BasicResponse, RequestProxy},
    HandleToken,
};
use crate::Error;
use futures::StreamExt;
use futures::TryFutureExt;
use serde::de::DeserializeOwned;

pub(crate) async fn call_request_method<R, B>(
    proxy: &zbus::azync::Proxy<'_>,
    handle_token: &HandleToken,
    method_name: &str,
    body: &B,
) -> Result<R, Error>
where
    R: serde::de::DeserializeOwned + zvariant::Type,
    B: serde::ser::Serialize + zvariant::Type,
{
    let request = RequestProxy::from_unique_name(proxy.connection(), handle_token).await?;
    let (response, path) = futures::try_join!(
        request.receive_response::<R>().into_future(),
        proxy
            .call::<B, zvariant::OwnedObjectPath>(method_name, body)
            .into_future()
            .map_err(From::from),
    )?;
    assert_eq!(&path.into_inner(), request.inner().path());
    Ok(response)
}

pub(crate) async fn call_basic_response_method<B>(
    proxy: &zbus::azync::Proxy<'_>,
    handle_token: &HandleToken,
    method_name: &str,
    body: &B,
) -> Result<(), Error>
where
    B: serde::ser::Serialize + zvariant::Type,
{
    call_request_method::<BasicResponse, B>(proxy, handle_token, method_name, body).await?;
    Ok(())
}

pub(crate) async fn receive_signal<T: DeserializeOwned + zvariant::Type>(
    proxy: &zbus::azync::Proxy<'_>,
    signal_name: &'static str,
) -> Result<T, Error> {
    let mut stream = proxy.receive_signal(signal_name).await?;
    let message = stream.next().await.ok_or(Error::NoResponse)?;
    message.body::<T>().map_err(From::from)
}

pub(crate) async fn call_method<R, B>(
    proxy: &zbus::azync::Proxy<'_>,
    method_name: &str,
    body: &B,
) -> Result<R, Error>
where
    R: serde::de::DeserializeOwned + zvariant::Type,
    B: serde::ser::Serialize + zvariant::Type,
{
    proxy
        .call::<B, R>(method_name, body)
        .await
        .map_err(From::from)
}

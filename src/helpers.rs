use std::fmt::Debug;

use futures::StreamExt;
use serde::Deserialize;

use crate::desktop::{
    request::{BasicResponse, RequestProxy, Response},
    HandleToken,
};
use crate::Error;

pub(crate) async fn call_request_method<R, B>(
    proxy: &zbus::azync::Proxy<'_>,
    handle_token: &HandleToken,
    method_name: &str,
    body: &B,
) -> Result<R, Error>
where
    R: for<'de> Deserialize<'de> + zvariant::Type + Debug,
    B: serde::ser::Serialize + zvariant::Type + Debug,
{
    tracing::info!(
        "Calling a request method '{}:{}'",
        proxy.interface(),
        method_name
    );
    tracing::debug!("The body is: {:#?}", body);
    let request = RequestProxy::from_unique_name(proxy.connection(), handle_token).await?;
    // We don't use receive_response because we want to create the stream in advance
    tracing::info!(
        "Listening to signal 'Response' on '{}'",
        request.inner().interface()
    );
    let mut stream = request.inner().receive_signal("Response").await?;

    let (response, path) = futures::try_join!(
        async {
            let message = stream.next().await.ok_or(Error::NoResponse)?;
            tracing::info!(
                "Received signal 'Response' on '{}'",
                request.inner().interface()
            );
            let response = match message.body::<Response<R>>()? {
                Response::Err(e) => Err(e.into()),
                Response::Ok(r) => Ok(r),
            };

            tracing::debug!("Received response {:#?}", response);
            response as Result<_, Error>
        },
        async {
            let msg = proxy.call_method(method_name, body).await?;
            let path = msg.body::<zvariant::OwnedObjectPath>()?.into_inner();

            tracing::debug!("Received request path {}", path.as_str());
            Ok(path) as Result<zvariant::ObjectPath<'_>, Error>
        },
    )?;
    assert_eq!(&path, request.inner().path());
    Ok(response)
}

pub(crate) async fn call_basic_response_method<B>(
    proxy: &zbus::azync::Proxy<'_>,
    handle_token: &HandleToken,
    method_name: &str,
    body: &B,
) -> Result<(), Error>
where
    B: serde::ser::Serialize + zvariant::Type + Debug,
{
    call_request_method::<BasicResponse, B>(proxy, handle_token, method_name, body).await?;
    Ok(())
}

pub(crate) async fn receive_signal<R>(
    proxy: &zbus::azync::Proxy<'_>,
    signal_name: &'static str,
) -> Result<R, Error>
where
    R: for<'de> Deserialize<'de> + zvariant::Type + Debug,
{
    tracing::info!(
        "Listening to signal '{}' on '{}'",
        signal_name,
        proxy.interface()
    );
    let mut stream = proxy.receive_signal(signal_name).await?;
    let message = stream.next().await.ok_or(Error::NoResponse)?;
    tracing::info!(
        "Received signal '{}' on '{}'",
        signal_name,
        proxy.interface()
    );
    let content = message.body::<R>()?;
    tracing::debug!("With body {:#?}", content);
    Ok(content)
}

pub(crate) async fn call_method<R, B>(
    proxy: &zbus::azync::Proxy<'_>,
    method_name: &str,
    body: &B,
) -> Result<R, Error>
where
    R: for<'de> Deserialize<'de> + zvariant::Type,
    B: serde::ser::Serialize + zvariant::Type + Debug,
{
    tracing::info!("Calling method {}:{}", proxy.interface(), method_name);
    tracing::debug!("With body {:#?}", body);
    let msg = proxy.call_method(method_name, body).await?;
    let reply = msg.body::<R>()?;
    msg.disown_fds();

    Ok(reply)
}

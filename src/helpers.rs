use std::{
    ffi::OsStr,
    fmt::Debug,
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
};
use zbus::zvariant::{ObjectPath, OwnedObjectPath, Type};

use futures::StreamExt;
use serde::Deserialize;

use crate::desktop::{
    request::{BasicResponse, RequestProxy, Response},
    HandleToken,
};
use crate::Error;

pub(crate) async fn call_request_method<R, B>(
    proxy: &zbus::Proxy<'_>,
    handle_token: &HandleToken,
    method_name: &str,
    body: &B,
) -> Result<R, Error>
where
    R: for<'de> Deserialize<'de> + Type + Debug,
    B: serde::ser::Serialize + Type + Debug,
{
    #[cfg(feature = "log")]
    tracing::info!(
        "Calling a request method '{}:{}'",
        proxy.interface(),
        method_name
    );
    #[cfg(feature = "log")]
    tracing::debug!("The body is: {:#?}", body);
    let request = RequestProxy::from_unique_name(proxy.connection(), handle_token).await?;
    // We don't use receive_response because we want to create the stream in advance
    #[cfg(feature = "log")]
    tracing::info!(
        "Listening to signal 'Response' on '{}'",
        request.inner().interface()
    );
    let mut stream = request.inner().receive_signal("Response").await?;

    let (response, path) = futures::try_join!(
        async {
            let message = stream.next().await.ok_or(Error::NoResponse)?;
            #[cfg(feature = "log")]
            tracing::info!(
                "Received signal 'Response' on '{}'",
                request.inner().interface()
            );
            let response = match message.body::<Response<R>>()? {
                Response::Err(e) => Err(e.into()),
                Response::Ok(r) => Ok(r),
            };
            #[cfg(feature = "log")]
            tracing::debug!("Received response {:#?}", response);
            response as Result<_, Error>
        },
        async {
            let msg = proxy.call_method(method_name, body).await?;
            let path = msg.body::<OwnedObjectPath>()?.into_inner();

            #[cfg(feature = "log")]
            tracing::debug!("Received request path {}", path.as_str());
            Ok(path) as Result<ObjectPath<'_>, Error>
        },
    )?;
    assert_eq!(&path, request.inner().path());
    Ok(response)
}

pub(crate) async fn call_basic_response_method(
    proxy: &zbus::Proxy<'_>,
    handle_token: &HandleToken,
    method_name: &str,
    body: &(impl serde::ser::Serialize + Type + Debug),
) -> Result<(), Error> {
    call_request_method::<BasicResponse, _>(proxy, handle_token, method_name, body).await?;
    Ok(())
}

pub(crate) async fn receive_signal<R>(
    proxy: &zbus::Proxy<'_>,
    signal_name: &'static str,
) -> Result<R, Error>
where
    R: for<'de> Deserialize<'de> + Type + Debug,
{
    #[cfg(feature = "log")]
    tracing::info!(
        "Listening to signal '{}' on '{}'",
        signal_name,
        proxy.interface()
    );
    let mut stream = proxy.receive_signal(signal_name).await?;
    let message = stream.next().await.ok_or(Error::NoResponse)?;
    #[cfg(feature = "log")]
    tracing::info!(
        "Received signal '{}' on '{}'",
        signal_name,
        proxy.interface()
    );
    let content = message.body::<R>()?;
    #[cfg(feature = "log")]
    tracing::debug!("With body {:#?}", content);
    Ok(content)
}

pub(crate) async fn call_method<R, B>(
    proxy: &zbus::Proxy<'_>,
    method_name: &str,
    body: &B,
) -> Result<R, Error>
where
    R: for<'de> Deserialize<'de> + Type,
    B: serde::ser::Serialize + Type + Debug,
{
    #[cfg(feature = "log")]
    {
        tracing::info!("Calling method {}:{}", proxy.interface(), method_name);
        tracing::debug!("With body {:#?}", body);
    }
    let msg = proxy.call_method(method_name, body).await?;
    let reply = msg.body::<R>()?;
    msg.take_fds();

    Ok(reply)
}

// Some portals returns paths which are bytes and not a typical string
// as those might be null terminated. This might make sense to provide in form of a helper in zvariant
pub(crate) fn path_from_null_terminated(bytes: Vec<u8>) -> PathBuf {
    Path::new(OsStr::from_bytes(bytes.split_last().unwrap().1)).to_path_buf()
}

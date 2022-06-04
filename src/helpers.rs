use std::{
    ffi::OsStr,
    fmt::Debug,
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
};

#[cfg(feature = "async-std")]
use async_std::{fs::File, prelude::*};
use futures::StreamExt;
use serde::Deserialize;
#[cfg(feature = "tokio")]
use tokio::{fs::File, io::AsyncReadExt};
use zbus::zvariant::{ObjectPath, OwnedObjectPath, Type};

use crate::{
    desktop::{
        request::{BasicResponse, RequestProxy, Response},
        HandleToken,
    },
    Error, PortalError,
};

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
    #[cfg(feature = "tracing")]
    tracing::info!(
        "Calling a request method '{}:{}'",
        proxy.interface(),
        method_name
    );
    #[cfg(feature = "tracing")]
    tracing::debug!("The body is: {:#?}", body);
    let request = RequestProxy::from_unique_name(proxy.connection(), handle_token).await?;
    // We don't use receive_response because we want to create the stream in advance
    #[cfg(feature = "tracing")]
    tracing::info!(
        "Listening to signal 'Response' on '{}'",
        request.inner().interface()
    );
    let mut stream = request
        .inner()
        .receive_signal("Response")
        .await
        .map_err::<PortalError, _>(From::from)?;

    let (response, path) = futures::try_join!(
        async {
            let message = stream.next().await.ok_or(Error::NoResponse)?;
            #[cfg(feature = "tracing")]
            tracing::info!(
                "Received signal 'Response' on '{}'",
                request.inner().interface()
            );
            let response = match message.body::<Response<R>>()? {
                Response::Err(e) => Err(e.into()),
                Response::Ok(r) => Ok(r),
            };
            #[cfg(feature = "tracing")]
            tracing::debug!("Received response {:#?}", response);
            response as Result<_, Error>
        },
        async {
            let msg = proxy
                .call_method(method_name, body)
                .await
                .map_err::<PortalError, _>(From::from)?;
            let path = msg.body::<OwnedObjectPath>()?.into_inner();

            #[cfg(feature = "tracing")]
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
    #[cfg(feature = "tracing")]
    tracing::info!(
        "Listening to signal '{}' on '{}'",
        signal_name,
        proxy.interface()
    );
    let mut stream = proxy
        .receive_signal(signal_name)
        .await
        .map_err::<PortalError, _>(From::from)?;
    let message = stream.next().await.ok_or(Error::NoResponse)?;
    #[cfg(feature = "tracing")]
    tracing::info!(
        "Received signal '{}' on '{}'",
        signal_name,
        proxy.interface()
    );
    let content = message.body::<R>()?;
    #[cfg(feature = "tracing")]
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
    #[cfg(feature = "tracing")]
    {
        tracing::info!("Calling method {}:{}", proxy.interface(), method_name);
        tracing::debug!("With body {:#?}", body);
    }
    let msg = proxy
        .call_method(method_name, body)
        .await
        .map_err::<PortalError, _>(From::from)?;
    let reply = msg.body::<R>()?;
    msg.take_fds();

    Ok(reply)
}

// Some portals returns paths which are bytes and not a typical string
// as those might be null terminated. This might make sense to provide in form
// of a helper in zvariant
pub(crate) fn path_from_null_terminated(bytes: Vec<u8>) -> PathBuf {
    Path::new(OsStr::from_bytes(bytes.split_last().unwrap().1)).to_path_buf()
}

pub(crate) async fn is_flatpak() -> bool {
    #[cfg(feature = "async-std")]
    {
        async_std::path::PathBuf::from("/.flatpak-info")
            .exists()
            .await
    }
    #[cfg(not(feature = "async-std"))]
    {
        std::path::PathBuf::from("/.flatpak-info").exists()
    }
}

pub(crate) async fn is_snap() -> bool {
    let pid = std::process::id();
    let path = format!("/proc/{}/cgroup", pid);
    let mut file = match File::open(path).await {
        Ok(file) => file,
        Err(_) => return false,
    };

    let mut buffer = String::new();
    match file.read_to_string(&mut buffer).await {
        Ok(_) => cgroup_v2_is_snap(&buffer),
        Err(_) => false,
    }
}

fn cgroup_v2_is_snap(cgroups: &str) -> bool {
    cgroups
        .lines()
        .map(|line| {
            let (n, rest) = line.split_once(':')?;
            // Check that n is a number.
            n.parse::<u32>().ok()?;
            let unit = match rest.split_once(':') {
                Some(("", unit)) => Some(unit),
                Some(("freezer", unit)) => Some(unit),
                Some(("name=systemd", unit)) => Some(unit),
                _ => None,
            }?;
            let scope = std::path::Path::new(unit).file_name()?.to_str()?;

            Some(scope.starts_with("snap."))
        })
        .any(|x| x.unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cgroup_v2_is_snap() {
        let data =
            "0::/user.slice/user-1000.slice/user@1000.service/apps.slice/snap.something.scope\n";
        assert_eq!(cgroup_v2_is_snap(data), true);

        let data = "0::/user.slice/user-1000.slice/user@1000.service/apps.slice\n";
        assert_eq!(cgroup_v2_is_snap(data), false);

        let data = "12:pids:/user.slice/user-1000.slice/user@1000.service
11:perf_event:/
10:net_cls,net_prio:/
9:cpuset:/
8:memory:/user.slice/user-1000.slice/user@1000.service/apps.slice/apps-org.gnome.Terminal.slice/vte-spawn-228ae109-a869-4533-8988-65ea4c10b492.scope
7:rdma:/
6:devices:/user.slice
5:blkio:/user.slice
4:hugetlb:/
3:freezer:/snap.portal-test
2:cpu,cpuacct:/user.slice
1:name=systemd:/user.slice/user-1000.slice/user@1000.service/apps.slice/apps-org.gnome.Terminal.slice/vte-spawn-228ae109-a869-4533-8988-65ea4c10b492.scope
0::/user.slice/user-1000.slice/user@1000.service/apps.slice/apps-org.gnome.Terminal.slice/vte-spawn-228ae109-a869-4533-8988-65ea4c10b492.scope\n";
        assert_eq!(cgroup_v2_is_snap(data), true);
    }
}

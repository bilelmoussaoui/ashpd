use super::{DESTINATION, PATH};
use crate::{helpers::call_method, Error};

/// Interface for setting a thread to realtime from within the sandbox.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Realtime`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Realtime).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Realtime")]
pub struct RealtimeProxy<'a>(zbus::Proxy<'a>);

impl<'a> RealtimeProxy<'a> {
    /// Create a new instance of [`RealtimeProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<RealtimeProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Realtime")?
            .path(PATH)?
            .destination(DESTINATION)?
            .build()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::Proxy<'_> {
        &self.0
    }

    #[doc(alias = "MakeThreadRealtimeWithPID")]
    pub async fn max_thread_realtime_with_pid(
        &self,
        process: u64,
        thread: u64,
        priority: u32,
    ) -> Result<(), Error> {
        call_method(
            self.inner(),
            "MakeThreadRealtimeWithPID",
            &(&process, &thread, &priority),
        )
        .await
    }

    #[doc(alias = "MakeThreadHighPriorityWithPID")]
    pub async fn max_thread_high_priority_with_pid(
        &self,
        process: u64,
        thread: u64,
        priority: u32,
    ) -> Result<(), Error> {
        call_method(
            self.inner(),
            "MakeThreadHighPriorityWithPID",
            &(&process, &thread, &priority),
        )
        .await
    }

    #[doc(alias = "MaxRealtimePriority")]
    pub async fn max_realtime_priority(&self) -> Result<i64, Error> {
        self.inner()
            .get_property::<i64>("MaxRealtimePriority")
            .await
            .map_err(From::from)
    }

    #[doc(alias = "MinNiceLevel")]
    pub async fn min_nice_level(&self) -> Result<u32, Error> {
        self.inner()
            .get_property::<u32>("MinNiceLevel")
            .await
            .map_err(From::from)
    }

    #[doc(alias = "RTTimeUSecMax")]
    pub async fn rt_time_usec_max(&self) -> Result<u32, Error> {
        self.inner()
            .get_property::<u32>("RTTimeUSecMax")
            .await
            .map_err(From::from)
    }
}

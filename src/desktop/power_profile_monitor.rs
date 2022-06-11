use super::{DESTINATION, PATH};
use crate::{helpers::session_connection, Error};

/// The interface provides information about the user-selected system-wide power
/// profile, to sandboxed applications.
///
/// It is not a portal in the strict sense,
/// since it does not involve user interaction.
///
/// Applications are expected to use this interface indirectly, via a library
/// API such as the GLib [`gio::PowerProfileMonitor`](https://gtk-rs.org/gtk-rs-core/stable/latest/docs/gio/struct.PowerProfileMonitor.html) interface.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.PowerProfileMonitor`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.PowerProfileMonitor).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.PowerProfileMonitor")]
pub struct PowerProfileMonitor<'a>(zbus::Proxy<'a>);

impl<'a> PowerProfileMonitor<'a> {
    /// Create a new instance of [`PowerProfileMonitor`].
    pub async fn new() -> Result<PowerProfileMonitor<'a>, Error> {
        let connection = session_connection().await?;
        let proxy = zbus::ProxyBuilder::new_bare(&connection)
            .interface("org.freedesktop.portal.PowerProfileMonitor")?
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

    /// Whether the power saver is enabled.
    #[doc(alias = "power-saver-enabled")]
    pub async fn is_enabled(&self) -> Result<bool, Error> {
        self.inner()
            .get_property::<bool>("power-saver-enabled")
            .await
            .map_err(From::from)
    }
}

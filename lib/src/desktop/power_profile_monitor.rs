use crate::{proxy::Proxy, Error};

/// The interface provides information about the user-selected system-wide power
/// profile, to sandboxed applications.
///
/// It is not a portal in the strict sense,
/// since it does not involve user interaction.
///
/// Applications are expected to use this interface indirectly, via a library
/// API such as the GLib [`gio::PowerProfileMonitor`](https://gtk-rs.org/gtk-rs-core/stable/latest/docs/gio/struct.PowerProfileMonitor.html) interface.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.PowerProfileMonitor`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.PowerProfileMonitor.html).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.PowerProfileMonitor")]
pub struct PowerProfileMonitor<'a>(Proxy<'a>);

impl<'a> PowerProfileMonitor<'a> {
    /// Create a new instance of [`PowerProfileMonitor`].
    pub async fn new() -> Result<PowerProfileMonitor<'a>, Error> {
        let proxy = Proxy::new_desktop("org.freedesktop.portal.PowerProfileMonitor").await?;
        Ok(Self(proxy))
    }

    /// Whether the power saver is enabled.
    ///
    /// # Specifications
    ///
    /// See also [`power-saver-enabled`](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.PowerProfileMonitor.html#org-freedesktop-portal-powerprofilemonitor-power-saver-enabled)
    #[doc(alias = "power-saver-enabled")]
    pub async fn is_enabled(&self) -> Result<bool, Error> {
        self.0.property("power-saver-enabled").await
    }
}

impl<'a> std::ops::Deref for PowerProfileMonitor<'a> {
    type Target = zbus::Proxy<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

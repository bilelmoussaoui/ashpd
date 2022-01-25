//! ```rust,no_run
//! use ashpd::desktop::settings::SettingsProxy;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = SettingsProxy::new(&connection).await?;
//!
//!     println!(
//!         "{:#?}",
//!         proxy
//!             .read::<String>("org.gnome.desktop.interface", "clock-format")
//!             .await?
//!     );
//!
//!     let settings = proxy.read_all(&["org.gnome.desktop.interface"]).await?;
//!     println!("{:#?}", settings);
//!
//!     let setting = proxy.receive_setting_changed().await?;
//!     println!("{}", setting.namespace());
//!     println!("{}", setting.key());
//!     println!("{:#?}", setting.value());
//!
//!     Ok(())
//! }
//! ```

use std::{collections::HashMap, convert::TryFrom, fmt::Debug};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use zbus::zvariant::{OwnedValue, Type};

use super::{DESTINATION, PATH};
use crate::{
    helpers::{call_method, receive_signal},
    Error,
};

/// A HashMap of the <key, value> settings found on a specific namespace.
pub type Namespace = HashMap<String, OwnedValue>;

#[derive(Serialize, Clone, Deserialize, Type)]
/// A specific `namespace.key = value` setting.
pub struct Setting(String, String, OwnedValue);

impl Setting {
    /// The setting namespace.
    pub fn namespace(&self) -> &str {
        &self.0
    }

    /// The setting key.
    pub fn key(&self) -> &str {
        &self.1
    }

    /// The setting value.
    pub fn value(&self) -> &OwnedValue {
        &self.2
    }
}

impl std::fmt::Debug for Setting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Setting")
            .field("namespace", &self.namespace())
            .field("key", &self.key())
            .field("value", self.value())
            .finish()
    }
}

/// The system's preferred color scheme
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorScheme {
    /// No preference
    NoPreference,
    /// Prefers dark appearance
    PreferDark,
    /// Prefers light appearance
    PreferLight,
}

/// The interface provides read-only access to a small number of host settings
/// required for toolkits similar to XSettings. It is not for general purpose
/// settings.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Settings`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Settings).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Settings")]
pub struct SettingsProxy<'a>(zbus::Proxy<'a>);

impl<'a> SettingsProxy<'a> {
    /// Create a new instance of [`SettingsProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<SettingsProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Settings")?
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

    /// Reads a single value. Returns an error on any unknown namespace or key.
    ///
    /// # Arguments
    ///
    /// * `namespaces` - List of namespaces to filter results by.
    ///
    /// If `namespaces` is an empty array or contains an empty string it matches
    /// all. Globing is supported but only for trailing sections, e.g.
    /// `org.example.*`.
    ///
    /// # Returns
    ///
    /// A `HashMap` of namespaces to its keys and values.
    ///
    /// # Specifications
    ///
    /// See also [`ReadAll`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Settings.ReadAll).
    #[doc(alias = "ReadAll")]
    pub async fn read_all(
        &self,
        namespaces: &[impl AsRef<str> + Type + Serialize + Debug],
    ) -> Result<HashMap<String, Namespace>, Error> {
        call_method(self.inner(), "ReadAll", &(namespaces)).await
    }

    /// Reads a single value. Returns an error on any unknown namespace or key.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace to look up key in.
    /// * `key` - The key to get.
    ///
    /// # Returns
    ///
    /// The value `key` is to to as a `zvariant::OwnedValue`.
    ///
    /// # Specifications
    ///
    /// See also [`Read`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Settings.Read).
    #[doc(alias = "Read")]
    pub async fn read<T>(&self, namespace: &str, key: &str) -> Result<T, Error>
    where
        T: TryFrom<OwnedValue> + DeserializeOwned + Type,
        Error: From<<T as TryFrom<OwnedValue>>::Error>,
    {
        let value = call_method::<OwnedValue, _>(self.inner(), "Read", &(namespace, key)).await?;
        T::try_from(value).map_err(From::from)
    }

    /// Reads the value of namespace: `org.freedesktop.appearance` and `color-scheme` key.
    pub async fn color_scheme(&self) -> Result<ColorScheme, Error> {
        let scheme = match self
            .read::<u32>("org.freedesktop.appearance", "color-scheme")
            .await?
        {
            1 => ColorScheme::PreferDark,
            2 => ColorScheme::PreferLight,
            _ => ColorScheme::NoPreference,
        };
        Ok(scheme)
    }

    /// Listen to changes of the namespace `org.freedesktop.appearance` for `color-scheme` key.
    pub async fn receive_color_scheme_changed(&self) -> Result<ColorScheme, Error> {
        loop {
            let setting = self.receive_setting_changed().await?;
            if setting.namespace() == "org.freedesktop.appearance"
                && setting.key() == "color-scheme"
            {
                return Ok(match u32::try_from(setting.value()) {
                    Ok(1) => ColorScheme::PreferDark,
                    Ok(2) => ColorScheme::PreferLight,
                    _ => ColorScheme::NoPreference,
                });
            }
        }
    }

    /// Signal emitted when a setting changes.
    ///
    /// # Specifications
    ///
    /// See also [`SettingChanged`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-signal-org-freedesktop-portal-Settings.SettingChanged).
    #[doc(alias = "SettingChanged")]
    pub async fn receive_setting_changed(&self) -> Result<Setting, Error> {
        receive_signal(self.inner(), "SettingChanged").await
    }
}

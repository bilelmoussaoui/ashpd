//! ```rust,no_run
//! use ashpd::desktop::settings::SettingsProxy;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
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

use std::{collections::HashMap, convert::TryFrom};

use futures::prelude::stream::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use zvariant::OwnedValue;
use zvariant_derive::Type;

use crate::{
    helpers::{call_method, property},
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

/// The interface provides read-only access to a small number of host settings
/// required for toolkits similar to XSettings. It is not for general purpose
/// settings.
#[derive(Debug)]
pub struct SettingsProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> SettingsProxy<'a> {
    /// Create a new instance of [`SettingsProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<SettingsProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Settings")
            .path("/org/freedesktop/portal/desktop")?
            .destination("org.freedesktop.portal.Desktop")
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Reads a single value. Returns an error on any unknown namespace or key.
    ///
    /// Returns a `HashMap` of namespaces to its keys and values.
    ///
    /// # Arguments
    ///
    /// * `namespaces` - List of namespaces to filter results by.
    ///
    /// If `namespaces` is an empty array or contains an empty string it matches
    /// all. Globing is supported but only for trailing sections, e.g.
    /// "org.example.*".
    pub async fn read_all(&self, namespaces: &[&str]) -> Result<HashMap<String, Namespace>, Error> {
        call_method(&self.0, "ReadAll", &(namespaces)).await
    }

    /// Reads a single value. Returns an error on any unknown namespace or key.
    ///
    /// Returns the value `key` is to to as a `zvariant::OwnedValue`.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace to look up key in.
    /// * `key` - The key to get.
    pub async fn read<T>(&self, namespace: &str, key: &str) -> Result<T, Error>
    where
        T: TryFrom<OwnedValue> + DeserializeOwned + zvariant::Type,
    {
        call_method(&self.0, "Read", &(namespace, key)).await
    }

    /// Signal emitted when a setting changes.
    pub async fn receive_setting_changed(&self) -> Result<Setting, Error> {
        let mut stream = self.0.receive_signal("SettingChanged").await?;
        let message = stream.next().await.ok_or(Error::NoResponse)?;
        message.body::<Setting>().map_err(From::from)
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}

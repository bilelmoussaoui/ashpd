//! ```rust,no_run
//! use ashpd::desktop::settings::SettingsProxy;
//! use zbus::{self, fdo::Result};
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = SettingsProxy::new(&connection);
//!
//!     println!(
//!         "{:#?}",
//!         proxy.read::<String>("org.gnome.desktop.interface", "clock-format")?
//!     );
//!     println!("{:#?}", proxy.read_all(&["org.gnome.desktop.interface"])?);
//!
//!     proxy.connect_setting_changed(|setting| {
//!         println!("{}", setting.namespace());
//!         println!("{}", setting.key());
//!         println!("{:#?}", setting.value());
//!         Ok(())
//!     })?;
//!     Ok(())
//! }
//! ```

use std::{collections::HashMap, convert::TryFrom};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::OwnedValue;
use zvariant_derive::Type;

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

#[dbus_proxy(
    interface = "org.freedesktop.portal.Desktop",
    default_service = "org.freedesktop.portal.Settings",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface provides read-only access to a small number of host settings
/// required for toolkits similar to XSettings. It is not for general purpose
/// settings.
trait Settings {
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
    fn read_all(&self, namespaces: &[&str]) -> zbus::Result<HashMap<String, Namespace>>;

    /// Reads a single value. Returns an error on any unknown namespace or key.
    ///
    /// Returns the value `key` is to to as a `zvariant::OwnedValue`.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace to look up key in.
    /// * `key` - The key to get.
    fn read<T>(&self, namespace: &str, key: &str) -> zbus::Result<T>
    where
        T: TryFrom<OwnedValue> + DeserializeOwned + zvariant::Type;

    #[dbus_proxy(signal)]
    /// Signal emitted when a setting changes.
    fn setting_changed(&self, setting: Setting) -> Result<()>;

    /// The version of this DBus interface.
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

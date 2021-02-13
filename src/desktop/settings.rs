//! ```no_run
//! use ashpd::desktop::settings::SettingsProxy;
//! use zbus::{self, fdo::Result};
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = SettingsProxy::new(&connection)?;
//!
//!     println!(
//!         "{:#?}",
//!         proxy.read("org.gnome.desktop.interface", "clock-format")?
//!     );
//!     println!("{:#?}", proxy.read_all(&["org.gnome.desktop.interface"])?);
//!
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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::OwnedValue;
use zvariant_derive::Type;

/// A HashMap of the <key, value> settings found on a specific namespace
pub type Namespace = HashMap<String, OwnedValue>;

#[derive(Debug, Serialize, Deserialize, Type)]
/// A specific namespace.key = value setting.
pub struct Setting(String, String, OwnedValue);

impl Setting {
    /// The setting namespace.
    pub fn namespace(&self) -> String {
        self.0.clone()
    }

    /// The setting key.
    pub fn key(&self) -> String {
        self.1.clone()
    }

    /// The setting value.
    pub fn value(&self) -> OwnedValue {
        self.2.clone()
    }
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Desktop",
    default_service = "org.freedesktop.portal.Settings",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface provides read-only access to a small number of host settings required for toolkits similar to XSettings.
/// It is not for general purpose settings.
trait Settings {
    /// Reads a single value. Returns an error on any unknown namespace or key.
    ///
    /// Returns a `HashMap` of namespaces to its keys and values.
    ///
    /// # Arguments
    ///
    /// * `namespaces` - List of namespaces to filter results by.
    ///
    ///     If `namespaces` is an empty array or contains an empty string it matches all.
    ///     Globbing is supported but only for trailing sections, e.g. "org.example.*".
    fn read_all(&self, namespaces: &[&str]) -> zbus::Result<HashMap<String, Namespace>>;

    /// Reads a single value. Returns an error on any unknown namespace or key.
    ///
    /// Returns the value `key` is to to as a `zvariant::OwnedValue`
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace to look up key in
    /// * `key` - The key to get
    fn read(&self, namespace: &str, key: &str) -> zbus::Result<zvariant::OwnedValue>;

    #[dbus_proxy(signal)]
    /// Signal emitted when a particular low memory situation happens with 0 being the lowest level of memory availability warning, and 255 being the highest
    fn setting_changed(&self, setting: Setting) -> Result<()>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

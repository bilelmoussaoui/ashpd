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
//!         proxy.read::<String>("org.gnome.desktop.interface", "clock-format")?
//!     );
//!     println!("{:#?}", proxy.read_all(&["org.gnome.desktop.interface"])?);
//!
//!
//!     proxy.on_setting_changed(|setting| {
//!         println!("{}", setting.namespace());
//!         println!("{}", setting.key());
//!         println!("{:#?}", setting.value());
//!     })?;
//!
//!
//!     Ok(())
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zbus::{fdo::Result, Connection, Proxy};
use zvariant::OwnedValue;
use zvariant_derive::Type;
use std::convert::TryFrom;

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

/// The interface provides read-only access to a small number of host settings required for toolkits similar to XSettings.
/// It is not for general purpose settings.
pub struct SettingsProxy<'a> {
    proxy: Proxy<'a>,
    connection: &'a Connection,
}

impl<'a> SettingsProxy<'a> {
    /// Creates a new settings proxy.
    pub fn new(connection: &'a Connection) -> Result<Self> {
        let proxy = Proxy::new(
            connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Settings",
        )?;
        Ok(Self { proxy, connection })
    }

    /// Signal emitted when a particular low memory situation happens with 0 being the lowest level of memory availability warning, and 255 being the highest
    pub fn on_setting_changed<F>(&self, callback: F) -> Result<()>
    where
        F: FnOnce(Setting),
    {
        loop {
            let msg = self.connection.receive_message()?;
            let msg_header = msg.header()?;
            if msg_header.message_type()? == zbus::MessageType::Signal
                && msg_header.member()? == Some("SettingChanged")
            {
                let response = msg.body::<Setting>()?;
                callback(response);
                break;
            }
        }
        Ok(())
    }

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
    pub fn read_all(&self, namespaces: &[&str]) -> zbus::Result<HashMap<String, Namespace>> {
        self.proxy.call("ReadAll", &(namespaces))
    }

    /// Reads a single value. Returns an error on any unknown namespace or key.
    ///
    /// Returns the value `key` is to to as a `zvariant::OwnedValue`
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace to look up key in
    /// * `key` - The key to get
    pub fn read<T: TryFrom<zvariant::OwnedValue>>(&self, namespace: &str, key: &str) -> zbus::Result<T> {
        self.proxy.call("Read", &(namespace, key))
    }

    /// version property
    pub fn version(&self) -> Result<u32> {
        self.proxy.get_property::<u32>("version")
    }
}

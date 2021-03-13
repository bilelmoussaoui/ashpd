//! **Note** this portal only works for sandboxed applications.
//!
//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::background::{Background, BackgroundOptions, BackgroundProxy};
//! use ashpd::{Response, WindowIdentifier};
//! use zbus::fdo::Result;
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = BackgroundProxy::new(&connection)?;
//!
//!     let request = proxy.request_background(
//!         WindowIdentifier::default(),
//!         BackgroundOptions::default()
//!             .autostart(true)
//!             .command(&["geary"])
//!             .reason("Automatically fetch your latest mails"),
//!     )?;
//!
//!     request.connect_response(|response: Response<Background>| {
//!         let bg = response.unwrap();
//!         println!("{}", bg.autostart);
//!         println!("{}", bg.background);
//!
//!         Ok(())
//!     })?;
//!     Ok(())
//! }
//! ```
use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{AsyncRequestProxy, HandleToken, RequestProxy, WindowIdentifier};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Clone, Default)]
/// Specified options for a `request_background` request.
pub struct BackgroundOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
    /// User-visible reason for the request.
    reason: Option<String>,
    /// `true` if the app also wants to be started automatically at login.
    autostart: Option<bool>,
    /// if `true`, use D-Bus activation for autostart.
    #[zvariant(rename = "dbus-activatable")]
    dbus_activatable: Option<bool>,
    /// Command to use when auto-starting at login.
    /// If this is not specified, the Exec line from the desktop file will be
    /// used.
    #[zvariant(rename = "commandline")]
    command: Option<Vec<String>>,
}

impl BackgroundOptions {
    /// Sets a user-visible reason for the request.
    pub fn reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }

    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    /// Sets whether to auto start the application or not.
    pub fn autostart(mut self, autostart: bool) -> Self {
        self.autostart = Some(autostart);
        self
    }

    /// Sets whether the application is dbus activatable.
    pub fn dbus_activatable(mut self, dbus_activatable: bool) -> Self {
        self.dbus_activatable = Some(dbus_activatable);
        self
    }

    /// Specifies the command line to execute.
    /// If this is not specified, the Exec line from the desktop file will be
    /// used.
    pub fn command(mut self, command: &[&str]) -> Self {
        let command = command.to_vec().iter().map(|s| s.to_string()).collect();
        self.command = Some(command);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// The response of a `request_background` request.
pub struct Background {
    /// If the application is allowed to run in the background.
    pub background: bool,
    /// If the application is will be auto-started.
    pub autostart: bool,
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Background",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications request that the application
/// is allowed to run in the background or started automatically when the user
/// logs in.
trait Background {
    /// Requests that the application is allowed to run in the background.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `options` - [`BackgroundOptions`].
    ///
    /// [`BackgroundOptions`]: ./struct.BackgroundOptions.html
    #[dbus_proxy(object = "Request")]
    fn request_background(&self, parent_window: WindowIdentifier, options: BackgroundOptions);

    /// The version of this DBus interface.
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

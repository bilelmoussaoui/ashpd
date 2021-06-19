//! **Note** this portal only works for sandboxed applications.
//!
//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::background::{Background, BackgroundOptions, BackgroundProxy};
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = BackgroundProxy::new(&connection).await?;
//!
//!     let response = proxy.request_background(
//!         WindowIdentifier::default(),
//!         BackgroundOptions::default()
//!             .autostart(true)
//!             .command(&["geary"])
//!             .reason("Automatically fetch your latest mails"),
//!     ).await?;
//!
//!     println!("{}", response.autostart);
//!     println!("{}", response.background);
//!
//!     Ok(())
//! }
//! ```
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{
    helpers::{call_request_method, property},
    Error, HandleToken, WindowIdentifier,
};

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

/// The interface lets sandboxed applications request that the application
/// is allowed to run in the background or started automatically when the user
/// logs in.
pub struct BackgroundProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> BackgroundProxy<'a> {
    pub async fn new(connection: &zbus::azync::Connection) -> Result<BackgroundProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Background")
            .path("/org/freedesktop/portal/desktop")?
            .destination("org.freedesktop.portal.Desktop")
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Requests that the application is allowed to run in the background.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.
    /// * `options` - [`BackgroundOptions`].
    ///
    /// [`BackgroundOptions`]: ./struct.BackgroundOptions.html
    pub async fn request_background(
        &self,
        parent_window: WindowIdentifier,
        options: BackgroundOptions,
    ) -> Result<Background, Error> {
        call_request_method(&self.0, "RequestBackground", &(parent_window, options)).await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}

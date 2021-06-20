//! **Note** this portal only works for sandboxed applications.
//!
//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::background::{Background, BackgroundProxy};
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = BackgroundProxy::new(&connection).await?;
//!
//!     let response = proxy
//!         .request_background(
//!             WindowIdentifier::default(),
//!             "Automatically fetch your latest mails",
//!             true,
//!             Some(&["geary"]),
//!             false
//!         )
//!         .await?;
//!
//!     println!("{}", response.auto_start());
//!     println!("{}", response.run_in_background());
//!
//!     Ok(())
//! }
//! ```
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

use crate::{helpers::call_request_method, Error, WindowIdentifier};

use super::{HandleToken, DESTINATION, PATH};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Clone, Default)]
/// Specified options for a [`BackgroundProxy::request_background`] request.
struct BackgroundOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
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
/// The response of a [`BackgroundProxy::request_background`] request.
pub struct Background {
    /// If the application is allowed to run in the background.
    background: bool,
    /// If the application is will be auto-started.
    autostart: bool,
}

impl Background {
    /// If the application is allowed to run in the background.
    pub fn run_in_background(&self) -> bool {
        self.background
    }

    /// If the application will be auto-started.
    pub fn auto_start(&self) -> bool {
        self.autostart
    }
}

/// The interface lets sandboxed applications request that the application
/// is allowed to run in the background or started automatically when the user
/// logs in.
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Background")]
pub struct BackgroundProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> BackgroundProxy<'a> {
    /// Create a new instance of [`BackgroundProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<BackgroundProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Background")
            .path(PATH)?
            .destination(DESTINATION)
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::azync::Proxy<'_> {
        &self.0
    }

    /// Requests that the application is allowed to run in the background.
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window.

    /// * `reason` - Sets a user-visible reason for the request.
    /// * `auto_start` - Sets whether to auto start the application or not.
    /// * `dbus_activatable` - Sets whether the application is dbus activatable.
    /// * `command_line` - Specifies the command line to execute.
    ///     If this is not specified, the Exec line from the desktop file will be
    ///     used.
    #[doc(alias = "RequestBackground")]
    pub async fn request_background(
        &self,
        parent_window: WindowIdentifier,
        reason: &str,
        auto_start: bool,
        command_line: Option<&[&str]>,
        dbus_activatable: bool,
    ) -> Result<Background, Error> {
        let options = BackgroundOptions::default()
            .reason(reason)
            .autostart(auto_start)
            .dbus_activatable(dbus_activatable)
            .command(&command_line.map(|t| t.to_vec()).unwrap_or_default());
        call_request_method(
            &self.0,
            &options.handle_token,
            "RequestBackground",
            &(parent_window, &options),
        )
        .await
    }
}

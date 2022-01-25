//! **Note** This portal only works for sandboxed applications.
//!
//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::background;
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let response = background::request(
//!         &WindowIdentifier::default(),
//!         "Automatically fetch your latest mails",
//!         true,
//!         Some(&["geary"]),
//!         false,
//!     )
//!     .await?;
//!
//!     println!("{}", response.auto_start());
//!     println!("{}", response.run_in_background());
//!
//!     Ok(())
//! }
//! ```
//!
//! If the [`None`] is provided as an argument for `command_line`, the [`Exec`](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#exec-variables) line from the [desktop
//! file](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#introduction) will be used.
//!
//! ```rust,no_run
//! use ashpd::desktop::background;
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let response = background::request(
//!         &WindowIdentifier::default(),
//!         "Automatically fetch your latest mails",
//!         true,
//!         None::<&[&str]>,
//!         false,
//!     )
//!     .await?;
//!
//!     println!("{}", response.auto_start());
//!     println!("{}", response.run_in_background());
//!
//!     Ok(())
//! }
//! ```

use serde::Serialize;
use zbus::zvariant::{DeserializeDict, SerializeDict, Type};

use super::{HandleToken, DESTINATION, PATH};
use crate::{helpers::call_request_method, Error, WindowIdentifier};

#[derive(SerializeDict, DeserializeDict, Type, Debug, Clone, Default)]
/// Specified options for a [`BackgroundProxy::request_background`] request.
#[zvariant(signature = "dict")]
struct BackgroundOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// User-visible reason for the request.
    reason: Option<String>,
    /// [`true`] if the app also wants to be started automatically at login.
    autostart: Option<bool>,
    /// if [`true`], use D-Bus activation for autostart.
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
    /// If this is not specified, the [`Exec`](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#exec-variables) line from the [desktop
    /// file](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#introduction)
    pub fn command(mut self, command: Option<&[impl AsRef<str> + Type + Serialize]>) -> Self {
        self.command = command.map(|s| s.iter().map(|s| s.as_ref().to_string()).collect());
        self
    }
}

#[derive(SerializeDict, DeserializeDict, Type, Debug)]
/// The response of a [`BackgroundProxy::request_background`] request.
#[zvariant(signature = "dict")]
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
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Background`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Background).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Background")]
pub struct BackgroundProxy<'a>(zbus::Proxy<'a>);

impl<'a> BackgroundProxy<'a> {
    /// Create a new instance of [`BackgroundProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<BackgroundProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Background")?
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

    /// Requests that the application is allowed to run in the background.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    /// * `reason` - Sets a user-visible reason for the request.
    /// * `auto_start` - Sets whether to auto start the application or not.
    /// * `command_line` - Specifies the command line to execute. If this is not
    ///   specified, the [`Exec`](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#exec-variables)
    ///   line from the [desktop
    /// file](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#introduction)
    /// * `dbus_activatable` - Sets whether the application is dbus activatable.
    ///
    /// # Specifications
    ///
    /// See also [`RequestBackground`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Background.RequestBackground).
    #[doc(alias = "RequestBackground")]
    pub async fn request_background(
        &self,
        identifier: &WindowIdentifier,
        reason: &str,
        auto_start: bool,
        command_line: Option<&[impl AsRef<str> + Type + Serialize]>,
        dbus_activatable: bool,
    ) -> Result<Background, Error> {
        let options = BackgroundOptions::default()
            .reason(reason)
            .autostart(auto_start)
            .dbus_activatable(dbus_activatable)
            .command(command_line);
        call_request_method(
            self.inner(),
            &options.handle_token,
            "RequestBackground",
            &(&identifier, &options),
        )
        .await
    }
}

#[doc(alias = "xdp_portal_request_background")]
/// A handy wrapper around [`BackgroundProxy::request_background`].
pub async fn request(
    identifier: &WindowIdentifier,
    reason: &str,
    auto_start: bool,
    command_line: Option<&[impl AsRef<str> + Type + Serialize]>,
    dbus_activatable: bool,
) -> Result<Background, Error> {
    let connection = zbus::Connection::session().await?;
    let proxy = BackgroundProxy::new(&connection).await?;
    proxy
        .request_background(
            identifier,
            reason,
            auto_start,
            command_line,
            dbus_activatable,
        )
        .await
}

//! The interface lets sandboxed applications request that the application
//! is allowed to run in the background or started automatically when the user
//! logs in.
//!
//! **Note** This portal only works for sandboxed applications.
//!
//! Wrapper of the DBus interface: [`org.freedesktop.portal.Background`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Background).
//!
//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::background::BackgroundRequest;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let response = BackgroundRequest::default()
//!         .reason("Automatically fetch your latest mails")
//!         .auto_start(true)
//!         .command_line(&["geary"])
//!         .dbus_activatable(false)
//!         .build()
//!         .await?;
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
//! use ashpd::desktop::background::BackgroundRequest;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let response = BackgroundRequest::default()
//!         .reason("Automatically fetch your latest mails")
//!         .auto_start(true)
//!         .build()
//!         .await?;
//!
//!     println!("{}", response.auto_start());
//!     println!("{}", response.run_in_background());
//!
//!     Ok(())
//! }
//! ```

use zbus::zvariant::{DeserializeDict, SerializeDict, Type};

use super::{HandleToken, DESTINATION, PATH};
use crate::{
    helpers::{call_request_method, session_connection},
    Error, WindowIdentifier,
};

#[derive(SerializeDict, Type, Debug, Default)]
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

#[derive(DeserializeDict, Type, Debug)]
/// The response of a [`BackgroundRequest`] request.
#[zvariant(signature = "dict")]
pub struct BackgroundResponse {
    background: bool,
    autostart: bool,
}

impl BackgroundResponse {
    /// Creates a new builder-pattern struct instance to construct
    /// [`BackgroundResponse`].
    ///
    /// This method returns an instance of [`BackgroundRequest`].
    pub fn builder() -> BackgroundRequest {
        BackgroundRequest::default()
    }

    /// If the application is allowed to run in the background.
    pub fn run_in_background(&self) -> bool {
        self.background
    }

    /// If the application will be auto-started.
    pub fn auto_start(&self) -> bool {
        self.autostart
    }
}

#[doc(alias = "org.freedesktop.portal.Background")]
struct BackgroundProxy<'a>(zbus::Proxy<'a>);

impl<'a> BackgroundProxy<'a> {
    /// Create a new instance of [`BackgroundProxy`].
    pub async fn new() -> Result<BackgroundProxy<'a>, Error> {
        let connection = session_connection().await?;
        let proxy = zbus::ProxyBuilder::new_bare(&connection)
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
        options: BackgroundOptions,
    ) -> Result<BackgroundResponse, Error> {
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
/// A [builder-pattern] type to construct [`BackgroundResponse`].
///
/// [builder-pattern]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
#[derive(Debug, Default)]
pub struct BackgroundRequest {
    identifier: WindowIdentifier,
    reason: Option<String>,
    auto_start: Option<bool>,
    dbus_activatable: Option<bool>,
    command_line: Option<Vec<String>>,
}

impl BackgroundRequest {
    #[must_use]
    /// Sets a window identifier.
    pub fn identifier(mut self, identifier: WindowIdentifier) -> Self {
        self.identifier = identifier;
        self
    }

    #[must_use]
    /// Sets whether to auto start the application or not.
    pub fn auto_start(mut self, auto_start: bool) -> Self {
        self.auto_start = Some(auto_start);
        self
    }

    #[must_use]
    /// Sets whether the application is dbus activatable.
    pub fn dbus_activatable(mut self, dbus_activatable: bool) -> Self {
        self.dbus_activatable = Some(dbus_activatable);
        self
    }

    #[must_use]
    /// Specifies the command line to execute.
    /// If this is not specified, the [`Exec`](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#exec-variables) line from the [desktop
    /// file](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#introduction)
    pub fn command_line(mut self, command_line: &[&str]) -> Self {
        self.command_line = Some(
            command_line
                .iter()
                .map(|s| s.to_owned().to_owned())
                .collect(),
        );
        self
    }

    #[must_use]
    /// Sets a user-visible reason for the request.
    pub fn reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_owned());
        self
    }

    /// Build the [`BackgroundResponse`].
    pub async fn build(self) -> Result<BackgroundResponse, Error> {
        let proxy = BackgroundProxy::new().await?;
        let options = BackgroundOptions {
            handle_token: Default::default(),
            reason: self.reason,
            autostart: self.auto_start,
            command: self.command_line,
            dbus_activatable: self.dbus_activatable,
        };
        proxy.request_background(&self.identifier, options).await
    }
}

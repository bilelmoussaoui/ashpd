//! # Examples
//!
//! How to inhibit logout/user switch
//!
//! ```rust,no_run
//! use ashpd::desktop::inhibit::{InhibitFlags, InhibitProxy, SessionState};
//! use ashpd::WindowIdentifier;
//! use std::{thread, time};
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = InhibitProxy::new(&connection).await?;
//!     let identifier = WindowIdentifier::default();
//!
//!     let session = proxy.create_monitor(&identifier).await?;
//!
//!     let state = proxy.receive_state_changed().await?;
//!     match state.session_state() {
//!         SessionState::Running => (),
//!         SessionState::QueryEnd => {
//!             proxy
//!                 .inhibit(
//!                     &identifier,
//!                     InhibitFlags::Logout | InhibitFlags::UserSwitch,
//!                     "please save the opened project first",
//!                 )
//!                 .await?;
//!             thread::sleep(time::Duration::from_secs(1));
//!             proxy.query_end_response(&session).await?;
//!         }
//!         SessionState::Ending => {
//!             println!("ending the session");
//!         }
//!     }
//!     Ok(())
//! }
//! ```

use enumflags2::{bitflags, BitFlags};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::zvariant::{DeserializeDict, OwnedObjectPath, SerializeDict, Type};

use super::{HandleToken, SessionProxy, DESTINATION, PATH};
use crate::{
    helpers::{call_basic_response_method, call_method, call_request_method, receive_signal},
    Error, WindowIdentifier,
};

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`InhibitProxy::create_monitor`] request.
#[zvariant(signature = "dict")]
struct CreateMonitorOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// A string that will be used as the last element of the session handle.
    session_handle_token: HandleToken,
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`InhibitProxy::inhibit`] request.
#[zvariant(signature = "dict")]
struct InhibitOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// User-visible reason for the inhibition.
    reason: Option<String>,
}

impl InhibitOptions {
    /// Sets a user visible reason for the inhibit request.
    pub fn reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }
}

#[bitflags]
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Copy, Type)]
#[repr(u32)]
#[doc(alias = "XdpInhibitFlags")]
/// The actions to inhibit that can end the user's session
pub enum InhibitFlags {
    #[doc(alias = "XDP_INHIBIT_FLAG_LOGOUT")]
    /// Logout.
    Logout,
    #[doc(alias = "XDP_INHIBIT_FLAG_USER_SWITCH")]
    /// User switch.
    UserSwitch,
    #[doc(alias = "XDP_INHIBIT_FLAG_SUSPEND")]
    /// Suspend.
    Suspend,
    #[doc(alias = "XDP_INHIBIT_FLAG_IDLE")]
    /// Idle.
    Idle,
}

#[derive(Debug, SerializeDict, DeserializeDict, Type)]
/// A response to a [`InhibitProxy::create_monitor`] request.
#[zvariant(signature = "dict")]
struct CreateMonitor {
    // TODO: investigate why this doesn't return an ObjectPath
    // replace with an ObjectPath once https://github.com/flatpak/xdg-desktop-portal/pull/609's merged
    session_handle: String,
}

#[derive(Debug, SerializeDict, DeserializeDict, Type)]
#[zvariant(signature = "dict")]
struct State {
    #[zvariant(rename = "screensaver-active")]
    screensaver_active: bool,
    #[zvariant(rename = "session-state")]
    session_state: SessionState,
}

#[derive(Debug, Serialize, Deserialize, Type)]
/// A response received when the `state_changed` signal is received.
pub struct InhibitState(OwnedObjectPath, State);

impl InhibitState {
    /// Whether screensaver is active or not.
    pub fn screensaver_active(&self) -> bool {
        self.1.screensaver_active
    }

    /// The session state.
    pub fn session_state(&self) -> SessionState {
        self.1.session_state
    }
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Copy, Type)]
#[doc(alias = "XdpLoginSessionState")]
#[repr(u32)]
/// The current state of the user's session.
pub enum SessionState {
    #[doc(alias = "XDP_LOGIN_SESSION_RUNNING")]
    /// Running.
    Running = 1,
    #[doc(alias = "XDP_LOGIN_SESSION_QUERY_END")]
    /// The user asked to end the session e.g logout.
    QueryEnd = 2,
    #[doc(alias = "XDP_LOGIN_SESSION_ENDING")]
    /// The session is ending.
    Ending = 3,
}

/// The interface lets sandboxed applications inhibit the user session from
/// ending, suspending, idling or getting switched away.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Inhibit`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Inhibit).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Inhibit")]
pub struct InhibitProxy<'a>(zbus::Proxy<'a>);

impl<'a> InhibitProxy<'a> {
    /// Create a new instance of [`InhibitProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<InhibitProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Inhibit")?
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

    /// Creates a monitoring session.
    /// While this session is active, the caller will receive `state_changed`
    /// signals with updates on the session state.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The application window identifier.
    ///
    /// # Specifications
    ///
    /// See also [`CreateMonitor`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Inhibit.CreateMonitor).
    #[doc(alias = "CreateMonitor")]
    #[doc(alias = "xdp_portal_session_monitor_start")]
    pub async fn create_monitor(
        &self,
        identifier: &WindowIdentifier,
    ) -> Result<SessionProxy<'a>, Error> {
        let options = CreateMonitorOptions::default();
        let body = &(&identifier, &options);
        let (monitor, proxy): (CreateMonitor, SessionProxy) = futures::try_join!(
            call_request_method(self.inner(), &options.handle_token, "CreateMonitor", body)
                .into_future(),
            SessionProxy::from_unique_name(
                self.inner().connection(),
                &options.session_handle_token
            )
            .into_future(),
        )?;
        assert_eq!(proxy.inner().path().as_str(), &monitor.session_handle);
        Ok(proxy)
    }

    /// Inhibits a session status changes.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The application window identifier.
    /// * `flags` - The flags determine what changes are inhibited.
    /// * `reason` - User-visible reason for the inhibition.
    ///
    /// # Specifications
    ///
    /// See also [`Inhibit`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Inhibit.Inhibit).
    #[doc(alias = "Inhibit")]
    #[doc(alias = "xdp_portal_session_inhibit")]
    pub async fn inhibit(
        &self,
        identifier: &WindowIdentifier,
        flags: BitFlags<InhibitFlags>,
        reason: &str,
    ) -> Result<(), Error> {
        let options = InhibitOptions::default().reason(reason);
        call_basic_response_method(
            self.inner(),
            &options.handle_token,
            "Inhibit",
            &(&identifier, flags, &options),
        )
        .await
    }

    /// Signal emitted when the session state changes.
    ///
    /// # Specifications
    ///
    /// See also [`StateChanged`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-signal-org-freedesktop-portal-Inhibit.StateChanged).
    #[doc(alias = "StateChanged")]
    #[doc(alias = "XdpPortal::session-state-changed")]
    pub async fn receive_state_changed(&self) -> Result<InhibitState, Error> {
        receive_signal(self.inner(), "StateChanged").await
    }

    /// Acknowledges that the caller received the "state_changed" signal.
    /// This method should be called within one second after receiving a
    /// [`receive_state_changed()`][`InhibitProxy::receive_state_changed`]
    /// signal with the [`SessionState::QueryEnd`] state.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_monitor()`][`InhibitProxy::create_monitor`].
    ///
    /// # Specifications
    ///
    /// See also [`QueryEndResponse`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-signal-org-freedesktop-portal-Inhibit.QueryEndResponse).
    #[doc(alias = "QueryEndResponse")]
    #[doc(alias = "xdp_portal_session_monitor_query_end_response")]
    pub async fn query_end_response(&self, session: &SessionProxy<'_>) -> Result<(), Error> {
        call_method(self.inner(), "QueryEndResponse", &(session)).await
    }
}

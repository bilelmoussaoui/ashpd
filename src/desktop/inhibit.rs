//! # Examples
//!
//! How to inhibit logout/user switch
//!
//! ```rust,no_run
//! use ashpd::desktop::inhibit::{InhibitFlags, InhibitProxy, SessionState};
//! use std::{thread, time};
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = InhibitProxy::new(&connection).await?;
//!
//!     let session = proxy.create_monitor(Default::default()).await?;
//!
//!     let state = proxy.receive_state_changed().await?;
//!     match state.session_state() {
//!         SessionState::Running => (),
//!         SessionState::QueryEnd => {
//!             proxy
//!                 .inhibit(
//!                     Default::default(),
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

use super::{HandleToken, SessionProxy, DESTINATION, PATH};
use crate::{
    helpers::{call_basic_response_method, call_method, call_request_method},
    Error, WindowIdentifier,
};
use enumflags2::BitFlags;
use futures::prelude::stream::*;
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zvariant::OwnedObjectPath;
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a [`InhibitProxy::create_monitor`] request.
struct CreateMonitorOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// A string that will be used as the last element of the session handle.
    session_handle_token: HandleToken,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a [`InhibitProxy::inhibit`] request.
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

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Copy, BitFlags, Type)]
#[repr(u32)]
/// The actions to inhibit that can end the user's session
pub enum InhibitFlags {
    /// Logout.
    Logout = 1,
    /// User switch.
    UserSwitch = 2,
    /// Suspend.
    Suspend = 4,
    /// Idle.
    Idle = 8,
}

#[derive(Debug, SerializeDict, DeserializeDict, TypeDict)]
/// A response to a [`InhibitProxy::create_monitor`] request.
struct CreateMonitor {
    session_handle: OwnedObjectPath,
}

#[derive(Debug, SerializeDict, DeserializeDict, TypeDict)]
#[doc(hidden)]
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
#[repr(u32)]
/// The current state of the user's session.
pub enum SessionState {
    /// Running.
    Running = 1,
    /// The user asked to end the session e.g logout.
    QueryEnd = 2,
    /// The session is ending.
    Ending = 3,
}

/// The interface lets sandboxed applications inhibit the user session from
/// ending, suspending, idling or getting switched away.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Inhibit`](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html#gdbus-org.freedesktop.portal.Inhibit).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Inhibit")]
pub struct InhibitProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> InhibitProxy<'a> {
    /// Create a new instance of [`InhibitProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<InhibitProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Inhibit")
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

    /// Creates a monitoring session.
    /// While this session is active, the caller will receive `state_changed`
    /// signals with updates on the session state.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The application window identifier.
    #[doc(alias = "CreateMonitor")]
    pub async fn create_monitor(
        &self,
        identifier: WindowIdentifier,
    ) -> Result<SessionProxy<'a>, Error> {
        let options = CreateMonitorOptions::default();
        let body = &(identifier, &options);
        let (monitor, proxy): (CreateMonitor, SessionProxy) = futures::try_join!(
            call_request_method(&self.0, &options.handle_token, "CreateMonitor", body)
                .into_future(),
            SessionProxy::from_unique_name(self.0.connection(), &options.session_handle_token)
                .into_future(),
        )?;
        assert_eq!(proxy.inner().path(), &monitor.session_handle.into_inner());
        Ok(proxy)
    }

    /// Inhibits a session status changes.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The application window identifier.
    /// * `flags` - The flags determine what changes are inhibited.
    /// * `reason` - User-visible reason for the inhibition.
    #[doc(alias = "Inhibit")]
    pub async fn inhibit(
        &self,
        identifier: WindowIdentifier,
        flags: BitFlags<InhibitFlags>,
        reason: &str,
    ) -> Result<(), Error> {
        let options = InhibitOptions::default().reason(reason);
        call_basic_response_method(
            &self.0,
            &options.handle_token,
            "Inhibit",
            &(identifier, flags, &options),
        )
        .await
    }

    /// Signal emitted when the session state changes.
    #[doc(alias = "StateChanged")]
    pub async fn receive_state_changed(&self) -> Result<InhibitState, Error> {
        let mut stream = self.0.receive_signal("StateChanged").await?;
        let message = stream.next().await.ok_or(Error::NoResponse)?;
        message.body::<InhibitState>().map_err(From::from)
    }

    /// Acknowledges that the caller received the "state_changed" signal.
    /// This method should be called within one second after receiving a
    /// `state_changed` signal with the `SessionState::QueryEnd` state.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`].
    #[doc(alias = "QueryEndResponse")]
    pub async fn query_end_response(&self, session: &SessionProxy<'_>) -> Result<(), Error> {
        call_method(&self.0, "QueryEndResponse", &(session)).await
    }
}

//! # Examples
//!
//! How to inhibit logout/user switch
//!
//! ```rust,no_run
//! use ashpd::desktop::inhibit::{
//!     CreateMonitorOptions, InhibitFlags, InhibitOptions, InhibitProxy, InhibitState, SessionState,
//! };
//! use ashpd::{HandleToken, WindowIdentifier};
//! use std::convert::TryFrom;
//! use std::{thread, time};
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = InhibitProxy::new(&connection).await?;
//!     let session_token = HandleToken::try_from("sessiontoken").unwrap();
//!     let session = proxy.create_monitor(
//!         WindowIdentifier::default(),
//!         CreateMonitorOptions::default().session_handle_token(session_token),
//!     ).await?;
//!
//!     let state = proxy.receive_state_changed().await?;
//!     match state.session_state() {
//!         SessionState::Running => (),
//!         SessionState::QueryEnd => {
//!             proxy.inhibit(
//!                 WindowIdentifier::default(),
//!                 InhibitFlags::Logout | InhibitFlags::UserSwitch,
//!                 InhibitOptions::default().reason("please save the opened project first"),
//!             ).await?;
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
use enumflags2::BitFlags;
use futures_lite::StreamExt;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zvariant::{ObjectPath, OwnedObjectPath};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

use crate::{Error, HandleToken, RequestProxy, SessionProxy, WindowIdentifier};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a create inhibit monitor request.
pub struct CreateMonitorOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
    /// A string that will be used as the last element of the session handle.
    session_handle_token: Option<HandleToken>,
}

impl CreateMonitorOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    /// Sets the session handle token.
    pub fn session_handle_token(mut self, session_handle_token: HandleToken) -> Self {
        self.session_handle_token = Some(session_handle_token);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options of an `inhibit` request.
pub struct InhibitOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
    /// User-visible reason for the inhibition.
    reason: Option<String>,
}

impl InhibitOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

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
/// A response to a `create_monitor` request.
struct CreateMonitor {
    pub(crate) session_handle: OwnedObjectPath,
}

#[derive(Debug, SerializeDict, DeserializeDict, TypeDict)]
#[doc(hidden)]
struct State {
    #[zvariant(rename = "screensaver-active")]
    pub screensaver_active: bool,
    #[zvariant(rename = "session-state")]
    pub session_state: SessionState,
}

#[derive(Debug, Serialize, Deserialize, Type)]
/// A response received when the `state_changed` signal is received.
pub struct InhibitState(OwnedObjectPath, State);

impl InhibitState {
    /// The session handle.
    pub fn session_handle(&self) -> &ObjectPath<'_> {
        &self.0
    }

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
pub struct InhibitProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> InhibitProxy<'a> {
    pub async fn new(connection: &zbus::azync::Connection) -> Result<InhibitProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Inhibit")
            .path("/org/freedesktop/portal/desktop")?
            .destination("org.freedesktop.portal.Desktop")
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Creates a monitoring session.
    /// While this session is active, the caller will receive `state_changed`
    /// signals with updates on the session state.
    ///
    /// # Arguments
    ///
    /// * `window` - The application window identifier.
    /// * `options` - [`CreateMonitorOptions`].
    ///
    /// [`CreateMonitorOptions`]: ./struct.CreateMonitorOptions.html
    pub async fn create_monitor(
        &self,
        window: WindowIdentifier,
        options: CreateMonitorOptions,
    ) -> Result<SessionProxy<'a>, Error> {
        let path: OwnedObjectPath = self
            .0
            .call_method("Inhibit", &(window, options))
            .await?
            .body()?;
        let request = RequestProxy::new(self.0.connection(), path).await?;
        let monitor = request.receive_response::<CreateMonitor>().await?;
        SessionProxy::new(self.0.connection(), monitor.session_handle).await
    }

    /// Inhibits a session status changes.
    ///
    /// # Arguments
    ///
    /// * `window` - The application window identifier.
    /// * `flags` - The flags determine what changes are inhibited.
    /// * `options` - A [`InhibitOptions`].
    ///
    /// [`InhibitOptions`]: ./struct.InhibitOptions.html
    pub async fn inhibit(
        &self,
        window: WindowIdentifier,
        flags: BitFlags<InhibitFlags>,
        options: InhibitOptions,
    ) -> Result<RequestProxy<'a>, Error> {
        let path: zvariant::OwnedObjectPath = self
            .0
            .call_method("Inhibit", &(window, flags, options))
            .await?
            .body()?;
        RequestProxy::new(self.0.connection(), path).await
    }

    /// Signal emitted when the session state changes.
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
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    pub async fn query_end_response(&self, session: &SessionProxy<'_>) -> Result<(), Error> {
        self.0
            .call_method("QueryEndResponse", &(session))
            .await?
            .body()
            .map_err(From::from)
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        self.0
            .get_property::<u32>("version")
            .await
            .map_err(From::from)
    }
}

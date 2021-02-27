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
//! use zbus::{self, fdo::Result};
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = InhibitProxy::new(&connection)?;
//!     let session_token = HandleToken::try_from("sessiontoken").unwrap();
//!     proxy.create_monitor(
//!         WindowIdentifier::default(),
//!         CreateMonitorOptions::default().session_handle_token(session_token),
//!     )?;
//!     proxy.connect_state_changed(move |state: InhibitState| {
//!         match state.session_state() {
//!             SessionState::Running => (),
//!             SessionState::QueryEnd => {
//!                 proxy.inhibit(
//!                     WindowIdentifier::default(),
//!                     InhibitFlags::Logout | InhibitFlags::UserSwitch,
//!                     InhibitOptions::default().reason("please save the opened project first"),
//!                 )?;
//!                 thread::sleep(time::Duration::from_secs(1));
//!                 proxy.query_end_response(state.session_handle())?;
//!             }
//!             SessionState::Ending => {
//!                 println!("ending the session");
//!             }
//!         }
//!         Ok(())
//!     })?;
//!     Ok(())
//! }
//! ```
use crate::{AsyncRequestProxy, HandleToken, RequestProxy, WindowIdentifier};
use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{ObjectPath, OwnedObjectPath};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a create inhibit monitor request.
pub struct CreateMonitorOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<HandleToken>,
    /// A string that will be used as the last element of the session handle.
    pub session_handle_token: Option<HandleToken>,
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
/// Specified options of an inhibit request.
pub struct InhibitOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<HandleToken>,
    /// User-visible reason for the inhibition.
    pub reason: Option<String>,
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
struct InhibitMonitorResponse {
    pub session_handle: OwnedObjectPath,
}

#[derive(Debug, Serialize, Deserialize, Type)]
/// A response received when the session state signal is received.
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

#[derive(Debug, SerializeDict, DeserializeDict, TypeDict)]
struct State {
    #[zvariant(rename = "screensaver-active")]
    pub screensaver_active: bool,
    #[zvariant(rename = "session-state")]
    pub session_state: SessionState,
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

#[dbus_proxy(
    interface = "org.freedesktop.portal.Desktop",
    default_service = "org.freedesktop.portal.Inhibit",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications inhibit the user session from ending, suspending, idling or getting switched away.
trait Inhibit {
    /// Creates a monitoring session.
    /// While this session is active, the caller will receive `state_changed` signals
    /// with updates on the session state.
    ///
    /// Returns a [`RequestProxy`].
    ///
    /// # Arguments
    ///
    /// * `window` - The application window identifier
    /// * `options` - [`CreateMonitorOptions`]
    ///
    /// [`CreateMonitorOptions`]: ./struct.CreateMonitorOptions.html
    #[dbus_proxy(object = "Request")]
    fn create_monitor(&self, window: WindowIdentifier, options: CreateMonitorOptions);

    /// Inhibits a session status changes.
    ///
    /// Returns a [`RequestProxy`].
    ///
    /// # Arguments
    ///
    /// * `window` - The application window identifier
    /// * `flags` - The flags determine what changes are inhibited
    /// * `options` - [`InhibitOptions`]
    ///
    /// [`InhibitOptions`]: ./struct.InhibitOptions.html
    /// [`RequestProxy`]: ../../request/struct.RequestProxy.html
    #[dbus_proxy(object = "Request")]
    fn inhibit(
        &self,
        window: WindowIdentifier,
        flags: BitFlags<InhibitFlags>,
        options: InhibitOptions,
    );

    /// Signal emitted when the session state changes.
    #[dbus_proxy(signal)]
    fn state_changed(&self, state: InhibitState) -> Result<()>;

    /// Acknowledges that the caller received the "state_changed" signal
    /// This method should be called within one second after receiving a `state_changed` signal with the `SessionState::QueryEnd` state.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    fn query_end_response(&self, session_handle: &ObjectPath<'_>) -> zbus::Result<()>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

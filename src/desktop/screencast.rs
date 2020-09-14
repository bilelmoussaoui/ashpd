//! # Examples
//!
//! How to create a screen cast session & start it.
//! The portal is currently useless without pipewire & rust support.
//!
//! ```no_run
//! use ashpd::desktop::screencast::{
//!     CreateSession, CreateSessionOptions, CursorMode, ScreenCastProxy, SelectSourcesOptions,
//!     SourceType, StartCastOptions, Streams,
//! };
//! use ashpd::{BasicResponse as Basic, HandleToken, RequestProxy, Response, WindowIdentifier};
//! use zbus::{self, fdo::Result};
//! use zvariant::ObjectPath;
//! use enumflags2::BitFlags;
//! use std::convert::TryFrom;
//!
//! fn select_sources(
//!     session_handle: ObjectPath,
//!     proxy: &ScreenCastProxy,
//!     connection: &zbus::Connection,
//! ) -> Result<()> {
//!     let request_handle = proxy.select_sources(
//!         session_handle.clone(),
//!         SelectSourcesOptions::default()
//!             .multiple(true)
//!             .cursor_mode(BitFlags::from(CursorMode::Metadata))
//!             .types(SourceType::Monitor | SourceType::Window),
//!     )?;
//!
//!     let request = RequestProxy::new(&connection, &request_handle)?;
//!     request.on_response(move |response: Response<Basic>| {
//!         if response.is_ok() {
//!             start_cast(session_handle, proxy, connection).unwrap();
//!         }
//!     })?;
//!     Ok(())
//! }
//!
//! fn start_cast(
//!     session_handle: ObjectPath,
//!     proxy: &ScreenCastProxy,
//!     connection: &zbus::Connection,
//! ) -> Result<()> {
//!     let request_handle = proxy.start(
//!         session_handle,
//!         WindowIdentifier::default(),
//!         StartCastOptions::default(),
//!     )?;
//!     let request = RequestProxy::new(&connection, &request_handle)?;
//!     request.on_response(move |r: Response<Streams>| {
//!         r.unwrap().streams().iter().for_each(|stream| {
//!             println!("{}", stream.pipewire_node_id());
//!             println!("{:#?}", stream.properties());
//!         });
//!     })?;
//!     Ok(())
//! }
//!
//! fn main() -> Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = ScreenCastProxy::new(&connection)?;
//!
//!     let session_token = HandleToken::try_from("session120").unwrap();
//!
//!     let request_handle = proxy.create_session(
//!         CreateSessionOptions::default()
//!             .session_handle_token(session_token)
//!     )?;
//!     let request = RequestProxy::new(&connection, &request_handle)?;
//!
//!     request.on_response(|r: Response<CreateSession>| {
//!         match r {
//!             Ok(session) => select_sources(session.handle(), &proxy, &connection).unwrap(),
//!             Err(_) => println!("hello!"),
//!         };
//!     })?;
//!     Ok(())
//! }
//! ```
use crate::{HandleToken, WindowIdentifier};
use core::convert::TryFrom;
use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{Fd, ObjectPath, OwnedObjectPath, Value};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Copy, Clone, Debug, Type, BitFlags)]
#[repr(u32)]
/// A bit flag for the available sources to record.
pub enum SourceType {
    /// A monitor.
    Monitor = 1,
    /// A specific window
    Window = 2,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Copy, Clone, Type, BitFlags)]
#[repr(u32)]
/// A bit flag for the possible cursor modes.
pub enum CursorMode {
    /// The cursor is not part of the screen cast stream.
    Hidden = 1,
    /// The cursor is embedded as part of the stream buffers.
    Embedded = 2,
    /// The cursor is not part of the screen cast stream, but sent as PipeWire stream metadata.
    Metadata = 4,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a create a screencast session request.
pub struct CreateSessionOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<HandleToken>,
    /// A string that will be used as the last element of the session handle.
    pub session_handle_token: Option<HandleToken>,
}

impl CreateSessionOptions {
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
/// Specified options on a select sources request.
pub struct SelectSourcesOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<HandleToken>,
    /// What types of content to record.
    pub types: Option<BitFlags<SourceType>>,
    /// Whether to allow selecting multiple sources.
    pub multiple: Option<bool>,
    /// Determines how the cursor will be drawn in the screen cast stream.
    pub cursor_mode: Option<BitFlags<CursorMode>>,
}

impl SelectSourcesOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    /// Sets whether to allow selecting multiple sources.
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = Some(multiple);
        self
    }

    /// Sets how the cursor will be drawn on the screen cast stream.
    pub fn cursor_mode(mut self, cursor_mode: BitFlags<CursorMode>) -> Self {
        self.cursor_mode = Some(cursor_mode);
        self
    }

    /// Sets the types of content to record.
    pub fn types(mut self, types: BitFlags<SourceType>) -> Self {
        self.types = Some(types);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a start screencast request.
pub struct StartCastOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<HandleToken>,
}

impl StartCastOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// A response to the create session request.
pub struct CreateSession {
    /// A string that will be used as the last element of the session handle.
    session_handle: String,
}

impl CreateSession {
    /// The created session handle.
    pub fn handle(&self) -> ObjectPath {
        ObjectPath::try_from(self.session_handle.clone()).unwrap()
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// A response to start the streamcast request.
pub struct Streams {
    streams: Vec<Stream>,
}

impl Streams {
    /// The available streams.
    pub fn streams(&self) -> &Vec<Stream> {
        &self.streams
    }
}

#[derive(Serialize, Deserialize, Type, Debug)]
/// A pipewire stream.
pub struct Stream(u32, StreamProperties);

impl Stream {
    /// Thep pipewire stream node
    pub fn pipewire_node_id(&self) -> u32 {
        self.0
    }

    /// The stream properties.
    pub fn properties(&self) -> &StreamProperties {
        &self.1
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// The stream properties.
pub struct StreamProperties {
    /// A tuple consisting of the position (x, y) in the compositor coordinate space.
    /// Note that the position may not be equivalent to a position in a pixel coordinate space.
    /// Only available for monitor streams.
    pub position: Option<(i32, i32)>,
    /// A tuple consisting of (width, height).
    /// The size represents the size of the stream as it is displayed in the compositor coordinate space.
    /// Note that this size may not be equivalent to a size in a pixel coordinate space.
    /// The size may differ from the size of the stream.
    pub size: (i32, i32),
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.ScreenCast",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications create screen cast sessions.
trait ScreenCast {
    /// Create a screen cast session.
    ///
    /// Returns a [`RequestProxy`] object path.
    ///
    /// [`RequestProxy`]: ../request/struct.RequestProxy.html
    fn create_session(&self, options: CreateSessionOptions) -> Result<OwnedObjectPath>;

    /// Open a file descriptor to the PipeWire remote where the screen cast streams are available.
    ///
    /// Returns a file descriptor of an open PipeWire remote.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `options` - ?
    /// FIXME: figure out the options we can take here
    ///
    /// [`SessionProxy`]: ../session/struct.SessionProxy.html
    fn open_pipe_wire_remote(
        &self,
        session_handle: ObjectPath,
        options: HashMap<&str, Value>,
    ) -> Result<Fd>;

    /// Configure what the screen cast session should record.
    /// This method must be called before starting the session.
    ///
    /// Passing invalid input to this method will cause the session to be closed.
    /// An application may only attempt to select sources once per session.
    ///
    /// Returns a [`RequestProxy`] object path.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `options` - A `SelectSourcesOptions`
    ///
    /// [`RequestProxy`]: ../request/struct.RequestProxy.html
    /// [`SessionProxy`]: ../session/struct.SessionProxy.html
    fn select_sources(
        &self,
        session_handle: ObjectPath,
        options: SelectSourcesOptions,
    ) -> Result<OwnedObjectPath>;

    /// Start the screen cast session.
    ///
    /// This will typically result the portal presenting a dialog letting the user do
    /// the selection set up by `select_sources`.
    ///
    /// An application can only attempt start a session once.
    ///
    /// Returns a [`RequestProxy`] object path.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `parent_window` - Identifier for the application window
    /// * `options` - A `StartScreenCastOptions`
    ///
    /// [`RequestProxy`]: ../request/struct.RequestProxy.html
    /// [`SessionProxy`]: ../session/struct.SessionProxy.html
    fn start(
        &self,
        session_handle: ObjectPath,
        parent_window: WindowIdentifier,
        options: StartCastOptions,
    ) -> Result<OwnedObjectPath>;

    /// Available cursor mode.
    #[dbus_proxy(property)]
    fn available_cursor_modes(&self) -> Result<u32>;

    /// Available source types.
    #[dbus_proxy(property)]
    fn available_source_types(&self) -> Result<u32>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

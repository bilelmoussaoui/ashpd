//! # Examples
//!
//! How to create a screen cast session & start it.
//! The portal is currently useless without PipeWire & Rust support.
//!
//! ```rust,no_run
//! use ashpd::desktop::screencast::{
//!     CreateSessionOptions, CursorMode, ScreenCastProxy, SelectSourcesOptions,
//!     SourceType, StartCastOptions, Streams,
//! };
//! use ashpd::{BasicResponse, HandleToken, SessionProxy, WindowIdentifier};
//! use enumflags2::BitFlags;
//! use std::convert::TryFrom;
//! use zvariant::ObjectPath;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = ScreenCastProxy::new(&connection).await?;
//!
//!     let session_token = HandleToken::try_from("session120").unwrap();
//!
//!     let session = proxy
//!         .create_session(CreateSessionOptions::default().session_handle_token(session_token)).await?;
//!
//!     let request = proxy.select_sources(
//!         &session,
//!         SelectSourcesOptions::default()
//!             .multiple(true)
//!             .cursor_mode(BitFlags::from(CursorMode::Metadata))
//!             .types(SourceType::Monitor | SourceType::Window),
//!     ).await?;
//!
//!     let _ = request.receive_response::<BasicResponse>().await?;
//!     let request = proxy.start(
//!         &session,
//!         WindowIdentifier::default(),
//!         StartCastOptions::default(),
//!     ).await?;
//!
//!     let response = request.receive_response::<Streams>().await?;
//!     response.streams().iter().for_each(|stream| {
//!         println!("{}", stream.pipe_wire_node_id());
//!         println!("{:#?}", stream.properties());
//!     });
//!     Ok(())
//! }
//! ```
use std::collections::HashMap;
use std::convert::TryFrom;

use enumflags2::BitFlags;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zvariant::{Fd, Value};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

use crate::{Error, HandleToken, RequestProxy, SessionProxy, WindowIdentifier};

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
    /// The cursor is not part of the screen cast stream, but sent as PipeWire
    /// stream metadata.
    Metadata = 4,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a create a Screen Cast session request.
pub struct CreateSessionOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
    /// A string that will be used as the last element of the session handle.
    session_handle_token: Option<HandleToken>,
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
    handle_token: Option<HandleToken>,
    /// What types of content to record.
    types: Option<BitFlags<SourceType>>,
    /// Whether to allow selecting multiple sources.
    multiple: Option<bool>,
    /// Determines how the cursor will be drawn in the screen cast stream.
    cursor_mode: Option<BitFlags<CursorMode>>,
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
/// Specified options on a start Screen Cast request.
pub struct StartCastOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
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
struct CreateSession {
    /// A string that will be used as the last element of the session handle.
    // TODO: investigate why this doesn't return an ObjectPath
    pub(crate) session_handle: String,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// A response to start the Stream Cast request.
pub struct Streams {
    streams: Vec<Stream>,
}

impl Streams {
    /// The available streams.
    pub fn streams(&self) -> &[Stream] {
        &self.streams
    }
}

#[derive(Serialize, Deserialize, Type, Debug, Clone)]
/// A PipeWire stream.
pub struct Stream(u32, StreamProperties);

impl Stream {
    /// The PipeWire stream Node ID
    pub fn pipe_wire_node_id(&self) -> u32 {
        self.0
    }

    /// The stream properties.
    pub fn properties(&self) -> &StreamProperties {
        &self.1
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Clone)]
/// The stream properties.
pub struct StreamProperties {
    /// A tuple consisting of the position (x, y) in the compositor coordinate
    /// space. **Note** that the position may not be equivalent to a
    /// position in a pixel coordinate space. Only available for monitor
    /// streams.
    pub position: Option<(i32, i32)>,
    /// A tuple consisting of (width, height).
    /// The size represents the size of the stream as it is displayed in the
    /// compositor coordinate space. **Note** that this size may not be
    /// equivalent to a size in a pixel coordinate space. The size may
    /// differ from the size of the stream.
    pub size: (i32, i32),
}

/// The interface lets sandboxed applications create screen cast sessions.
pub struct ScreenCastProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> ScreenCastProxy<'a> {
    pub async fn new(connection: &zbus::azync::Connection) -> Result<ScreenCastProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.ScreenCast")
            .path("/org/freedesktop/portal/desktop")?
            .destination("org.freedesktop.portal.Desktop")
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Create a screen cast session.
    pub async fn create_session(
        &self,
        options: CreateSessionOptions,
    ) -> Result<SessionProxy<'a>, Error> {
        let path: zvariant::OwnedObjectPath = self
            .0
            .call_method("CreateSession", &(options))
            .await?
            .body()?;
        let request = RequestProxy::new(self.0.connection(), path).await?;
        let session = request.receive_response::<CreateSession>().await?;
        SessionProxy::new(
            self.0.connection(),
            zvariant::OwnedObjectPath::try_from(session.session_handle)?,
        )
        .await
    }

    /// Open a file descriptor to the PipeWire remote where the screen cast
    /// streams are available.
    ///
    /// Returns a file descriptor of an open PipeWire remote.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`].
    /// * `options` - ?
    /// FIXME: figure out the options we can take here
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    pub async fn open_pipe_wire_remote(
        &self,
        session: &SessionProxy<'_>,
        options: HashMap<&str, Value<'_>>,
    ) -> Result<Fd, Error> {
        self.0
            .call_method("OpenPipeWireRemote", &(session, options))
            .await?
            .body()
            .map_err(From::from)
    }

    /// Configure what the screen cast session should record.
    /// This method must be called before starting the session.
    ///
    /// Passing invalid input to this method will cause the session to be
    /// closed. An application may only attempt to select sources once per
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`].
    /// * `options` - A `SelectSourcesOptions`.
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    pub async fn select_sources(
        &self,
        session: &SessionProxy<'_>,
        options: SelectSourcesOptions,
    ) -> Result<RequestProxy<'a>, Error> {
        let path: zvariant::OwnedObjectPath = self
            .0
            .call_method("SelectSources", &(session, options))
            .await?
            .body()?;
        RequestProxy::new(self.0.connection(), path).await
    }

    /// Start the screen cast session.
    ///
    /// This will typically result the portal presenting a dialog letting the
    /// user do the selection set up by `select_sources`.
    ///
    /// An application can only attempt start a session once.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`].
    /// * `parent_window` - Identifier for the application window.
    /// * `options` - A `StartScreenCastOptions`.
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    pub async fn start(
        &self,
        session: &SessionProxy<'_>,
        parent_window: WindowIdentifier,
        options: StartCastOptions,
    ) -> Result<RequestProxy<'a>, Error> {
        let path: zvariant::OwnedObjectPath = self
            .0
            .call_method("Start", &(session, parent_window, options))
            .await?
            .body()?;
        RequestProxy::new(self.0.connection(), path).await
    }

    /// Available cursor mode.
    pub async fn available_cursor_modes(&self) -> Result<BitFlags<CursorMode>, Error> {
        self.0
            .get_property::<BitFlags<CursorMode>>("AvailableCursorModes")
            .await
            .map_err(From::from)
    }

    /// Available source types.
    pub async fn available_source_types(&self) -> Result<BitFlags<SourceType>, Error> {
        self.0
            .get_property::<BitFlags<SourceType>>("AvailableSourceTypes")
            .await
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

//! # Examples
//!
//! How to create a screen cast session & start it.
//! The portal is currently useless without PipeWire & Rust support.
//!
//! ```rust,no_run
//! use ashpd::desktop::screencast::{CursorMode, ScreenCastProxy, SourceType};
//! use enumflags2::BitFlags;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = ScreenCastProxy::new(&connection).await?;
//!
//!     let session = proxy.create_session().await?;
//!
//!     proxy
//!         .select_sources(
//!             &session,
//!             BitFlags::from(CursorMode::Metadata),
//!             SourceType::Monitor | SourceType::Window,
//!             true,
//!         )
//!         .await?;
//!
//!     let streams = proxy.start(&session, Default::default()).await?;
//!
//!     streams.iter().for_each(|stream| {
//!         println!("node id: {}", stream.pipe_wire_node_id());
//!         println!("size: {:?}", stream.size());
//!         println!("position: {:?}", stream.position());
//!     });
//!     Ok(())
//! }
//! ```

use enumflags2::BitFlags;
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use zvariant::{Fd, Value};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

use super::{HandleToken, SessionProxy, DESTINATION, PATH};
use crate::{
    helpers::{call_basic_response_method, call_method, call_request_method, property},
    Error, WindowIdentifier,
};

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
/// Specified options for a [`ScreenCastProxy::create_session`] request.
struct CreateSessionOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// A string that will be used as the last element of the session handle.
    session_handle_token: HandleToken,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a [`ScreenCastProxy::select_sources`] request.
struct SelectSourcesOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// What types of content to record.
    types: Option<BitFlags<SourceType>>,
    /// Whether to allow selecting multiple sources.
    multiple: Option<bool>,
    /// Determines how the cursor will be drawn in the screen cast stream.
    cursor_mode: Option<BitFlags<CursorMode>>,
}

impl SelectSourcesOptions {
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
/// Specified options for a [`ScreenCastProxy::start`] request.
struct StartCastOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// A response to a [`ScreenCastProxy::create_session`] request.
struct CreateSession {
    /// A string that will be used as the last element of the session handle.
    // TODO: investigate why this doesn't return an ObjectPath
    session_handle: String,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// A response to a [`ScreenCastProxy::start`] request.
struct Streams {
    pub streams: Vec<Stream>,
}

#[derive(Serialize, Deserialize, Type, Debug, Clone)]
/// A PipeWire stream.
pub struct Stream(u32, StreamProperties);

impl Stream {
    /// The PipeWire stream Node ID
    pub fn pipe_wire_node_id(&self) -> u32 {
        self.0
    }

    /// A tuple consisting of the position (x, y) in the compositor coordinate
    /// space.
    ///
    ///  **Note** that the position may not be equivalent to a
    /// position in a pixel coordinate space. Only available for monitor
    /// streams.
    pub fn position(&self) -> Option<(i32, i32)> {
        self.1.position
    }

    /// A tuple consisting of (width, height).
    /// The size represents the size of the stream as it is displayed in the
    /// compositor coordinate space.
    ///
    /// **Note** that this size may not be
    /// equivalent to a size in a pixel coordinate space. The size may
    /// differ from the size of the stream.
    pub fn size(&self) -> (i32, i32) {
        self.1.size
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Clone)]
/// The stream properties.
struct StreamProperties {
    position: Option<(i32, i32)>,
    size: (i32, i32),
}

/// The interface lets sandboxed applications create screen cast sessions.
#[derive(Debug)]
pub struct ScreenCastProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> ScreenCastProxy<'a> {
    /// Create a new instance of [`ScreenCastProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<ScreenCastProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.ScreenCast")
            .path(PATH)?
            .destination(DESTINATION)
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Create a screen cast session.
    pub async fn create_session(&self) -> Result<SessionProxy<'a>, Error> {
        let options = CreateSessionOptions::default();
        let (proxy, session) = futures::try_join!(
            SessionProxy::from_unique_name(self.0.connection(), &options.session_handle_token)
                .into_future(),
            call_request_method::<CreateSession, CreateSessionOptions>(
                &self.0,
                &options.handle_token,
                "CreateSession",
                &(&options)
            )
            .into_future()
        )?;
        assert_eq!(proxy.inner().path().to_string(), session.session_handle);
        Ok(proxy)
    }

    /// Open a file descriptor to the PipeWire remote where the screen cast
    /// streams are available.
    ///
    /// Returns a file descriptor of an open PipeWire remote.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`].
    pub async fn open_pipe_wire_remote(&self, session: &SessionProxy<'_>) -> Result<Fd, Error> {
        // FIXME: figure out the options we can take here
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(&self.0, "OpenPipeWireRemote", &(session, options)).await
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
    /// * `cursor_mode` - Sets how the cursor will be drawn on the screen cast stream.
    /// * `types` - Sets the types of content to record.
    /// * `multiple`- Sets whether to allow selecting multiple sources.
    pub async fn select_sources(
        &self,
        session: &SessionProxy<'_>,
        cursor_mode: BitFlags<CursorMode>,
        types: BitFlags<SourceType>,
        multiple: bool,
    ) -> Result<(), Error> {
        let options = SelectSourcesOptions::default()
            .cursor_mode(cursor_mode)
            .multiple(multiple)
            .types(types);
        call_basic_response_method(
            &self.0,
            &options.handle_token,
            "SelectSources",
            &(session, &options),
        )
        .await
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
    pub async fn start(
        &self,
        session: &SessionProxy<'_>,
        parent_window: WindowIdentifier,
    ) -> Result<Vec<Stream>, Error> {
        let options = StartCastOptions::default();
        let streams: Streams = call_request_method(
            &self.0,
            &options.handle_token,
            "Start",
            &(session, parent_window, &options),
        )
        .await?;
        Ok(streams.streams.to_vec())
    }

    /// Available cursor mode.
    pub async fn available_cursor_modes(&self) -> Result<BitFlags<CursorMode>, Error> {
        property(&self.0, "AvailableCursorModes").await
    }

    /// Available source types.
    pub async fn available_source_types(&self) -> Result<BitFlags<SourceType>, Error> {
        property(&self.0, "AvailableSourceTypes").await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}

//! # Examples
//!
//! How to create a screen cast session & start it.
//! The portal is currently useless without PipeWire & Rust support.
//!
//! ```rust,no_run
//! use ashpd::desktop::screencast::{CursorMode, PersistMode, ScreenCastProxy, SourceType};
//! use ashpd::WindowIdentifier;
//! use enumflags2::BitFlags;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
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
//!             None,
//!             PersistMode::DoNot,
//!         )
//!         .await?;
//!
//!     let (streams, token) = proxy.start(&session, &WindowIdentifier::default()).await?;
//!
//!     streams.iter().for_each(|stream| {
//!         println!("node id: {}", stream.pipe_wire_node_id());
//!         println!("size: {:?}", stream.size());
//!         println!("position: {:?}", stream.position());
//!     });
//!     Ok(())
//! }
//! ```

use std::{
    collections::HashMap,
    fmt::Debug,
    os::unix::prelude::{IntoRawFd, RawFd},
};

use enumflags2::{bitflags, BitFlags};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::zvariant::{DeserializeDict, OwnedFd, SerializeDict, Type, Value};

use super::{HandleToken, SessionProxy, DESTINATION, PATH};
use crate::{
    helpers::{call_basic_response_method, call_method, call_request_method},
    Error, WindowIdentifier,
};

#[bitflags]
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Copy, Clone, Debug, Type)]
#[repr(u32)]
#[doc(alias = "XdpOutputType")]
/// A bit flag for the available sources to record.
pub enum SourceType {
    #[doc(alias = "XDP_OUTPUT_MONITOR")]
    /// A monitor.
    Monitor,
    #[doc(alias = "XDP_OUTPUT_WINDOW")]
    /// A specific window
    Window,
    #[doc(alias = "XDP_OUTPUT_VIRTUAL")]
    /// Virtual
    Virtual,
}

#[bitflags]
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Copy, Clone, Type)]
#[repr(u32)]
#[doc(alias = "XdpCursorMode")]
/// A bit flag for the possible cursor modes.
pub enum CursorMode {
    #[doc(alias = "XDP_CURSOR_MODE_HIDDEN")]
    /// The cursor is not part of the screen cast stream.
    Hidden,
    #[doc(alias = "XDP_CURSOR_MODE_EMBEDDED")]
    /// The cursor is embedded as part of the stream buffers.
    Embedded,
    #[doc(alias = "XDP_CURSOR_MODE_METADATA")]
    /// The cursor is not part of the screen cast stream, but sent as PipeWire
    /// stream metadata.
    Metadata,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Copy, Clone, Type)]
#[doc(alias = "XdpPersistMode")]
#[repr(u32)]
pub enum PersistMode {
    #[doc(alias = "XDP_PERSIST_MODE_NONE")]
    /// Do not persist.
    DoNot = 0,
    #[doc(alias = "XDP_PERSIST_MODE_TRANSIENT")]
    /// Persist while the application is running.
    Application = 1,
    #[doc(alias = "XDP_PERSIST_MODE_PERSISTENT")]
    /// Persist until explicitly revoked.
    ExplicitlyRevoked = 2,
}

impl Default for PersistMode {
    fn default() -> Self {
        Self::DoNot
    }
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`ScreenCastProxy::create_session`] request.
#[zvariant(signature = "dict")]
struct CreateSessionOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// A string that will be used as the last element of the session handle.
    session_handle_token: HandleToken,
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`ScreenCastProxy::select_sources`] request.
#[zvariant(signature = "dict")]
struct SelectSourcesOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// What types of content to record.
    types: Option<BitFlags<SourceType>>,
    /// Whether to allow selecting multiple sources.
    multiple: Option<bool>,
    /// Determines how the cursor will be drawn in the screen cast stream.
    cursor_mode: Option<BitFlags<CursorMode>>,
    restore_token: Option<String>,
    persist_mode: Option<PersistMode>,
}

impl SelectSourcesOptions {
    /// Sets whether to allow selecting multiple sources.
    #[must_use]
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = Some(multiple);
        self
    }

    /// Sets how the cursor will be drawn on the screen cast stream.
    #[must_use]
    pub fn cursor_mode(mut self, cursor_mode: BitFlags<CursorMode>) -> Self {
        self.cursor_mode = Some(cursor_mode);
        self
    }

    /// Sets the types of content to record.
    #[must_use]
    pub fn types(mut self, types: BitFlags<SourceType>) -> Self {
        self.types = Some(types);
        self
    }

    #[must_use]
    pub fn persist_mode(mut self, persist_mode: PersistMode) -> Self {
        self.persist_mode = Some(persist_mode);
        self
    }

    pub fn set_restore_token(&mut self, token: &str) {
        self.restore_token = Some(token.to_string());
    }
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`ScreenCastProxy::start`] request.
#[zvariant(signature = "dict")]
struct StartCastOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
}

#[derive(SerializeDict, DeserializeDict, Type, Debug)]
/// A response to a [`ScreenCastProxy::create_session`] request.
#[zvariant(signature = "dict")]
struct CreateSession {
    // TODO: investigate why this doesn't return an ObjectPath
    // replace with an ObjectPath once https://github.com/flatpak/xdg-desktop-portal/pull/609's merged
    /// A string that will be used as the last element of the session handle.
    session_handle: String,
}

#[derive(SerializeDict, DeserializeDict, Type)]
/// A response to a [`ScreenCastProxy::start`] request.
#[zvariant(signature = "dict")]
struct Streams {
    streams: Vec<Stream>,
    restore_token: Option<String>,
}

impl Debug for Streams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.streams.iter()).finish()
    }
}

#[derive(Serialize, Deserialize, Type, Clone)]
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
    /// **Note** the position may not be equivalent to a position in a pixel
    /// coordinate space. Only available for monitor streams.
    pub fn position(&self) -> Option<(i32, i32)> {
        self.1.position
    }

    /// A tuple consisting of (width, height).
    /// The size represents the size of the stream as it is displayed in the
    /// compositor coordinate space.
    ///
    /// **Note** the size may not be equivalent to a size in a pixel coordinate
    /// space. The size may differ from the size of the stream.
    pub fn size(&self) -> Option<(i32, i32)> {
        self.1.size
    }

    pub fn source_type(&self) -> Option<SourceType> {
        self.1.source_type
    }

    pub fn id(&self) -> Option<&str> {
        self.1.id.as_deref()
    }
}

impl Debug for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stream")
            .field("pipewire_node_id", &self.pipe_wire_node_id())
            .field("position", &self.position())
            .field("size", &self.size())
            .field("source_type", &self.source_type())
            .field("id", &self.id())
            .finish()
    }
}
#[derive(SerializeDict, DeserializeDict, Type, Debug, Clone)]
/// The stream properties.
#[zvariant(signature = "dict")]
struct StreamProperties {
    id: Option<String>,
    position: Option<(i32, i32)>,
    size: Option<(i32, i32)>,
    source_type: Option<SourceType>,
}

/// The interface lets sandboxed applications create screen cast sessions.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.ScreenCast`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.ScreenCast).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.ScreenCast")]
pub struct ScreenCastProxy<'a>(zbus::Proxy<'a>);

impl<'a> ScreenCastProxy<'a> {
    /// Create a new instance of [`ScreenCastProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<ScreenCastProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.ScreenCast")?
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

    /// Create a screen cast session.
    ///
    /// # Specifications
    ///
    /// See also [`CreateSession`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-ScreenCast.CreateSession).
    #[doc(alias = "CreateSession")]
    #[doc(alias = "xdp_portal_create_screencast_session")]
    pub async fn create_session(&self) -> Result<SessionProxy<'a>, Error> {
        let options = CreateSessionOptions::default();
        let (session, proxy) = futures::try_join!(
            call_request_method::<CreateSession, _>(
                self.inner(),
                &options.handle_token,
                "CreateSession",
                &options
            )
            .into_future(),
            SessionProxy::from_unique_name(
                self.inner().connection(),
                &options.session_handle_token
            )
            .into_future(),
        )?;
        assert_eq!(proxy.inner().path().as_str(), &session.session_handle);
        Ok(proxy)
    }

    /// Open a file descriptor to the PipeWire remote where the screen cast
    /// streams are available.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`ScreenCastProxy::create_session`].
    ///
    /// # Returns
    ///
    /// File descriptor of an open PipeWire remote.
    ///
    /// # Specifications
    ///
    /// See also [`OpenPipeWireRemote`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-ScreenCast.OpenPipeWireRemote).
    #[doc(alias = "OpenPipeWireRemote")]
    pub async fn open_pipe_wire_remote(&self, session: &SessionProxy<'_>) -> Result<RawFd, Error> {
        // `options` parameter doesn't seems to be used yet
        // see https://github.com/flatpak/xdg-desktop-portal/blob/master/src/screen-cast.c#L812
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let fd: OwnedFd =
            call_method(self.inner(), "OpenPipeWireRemote", &(session, options)).await?;
        Ok(fd.into_raw_fd())
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
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`ScreenCastProxy::create_session`].
    /// * `cursor_mode` - Sets how the cursor will be drawn on the screen cast
    ///   stream.
    /// * `types` - Sets the types of content to record.
    /// * `multiple`- Sets whether to allow selecting multiple sources.
    ///
    /// # Specifications
    ///
    /// See also [`SelectSources`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-ScreenCast.SelectSources).
    #[doc(alias = "SelectSources")]
    pub async fn select_sources(
        &self,
        session: &SessionProxy<'_>,
        cursor_mode: BitFlags<CursorMode>,
        types: BitFlags<SourceType>,
        multiple: bool,
        restore_token: Option<&str>,
        persist_mode: PersistMode,
    ) -> Result<(), Error> {
        let mut options = SelectSourcesOptions::default()
            .cursor_mode(cursor_mode)
            .multiple(multiple)
            .types(types)
            .persist_mode(persist_mode);
        if let Some(token) = restore_token {
            options.set_restore_token(token);
        }
        call_basic_response_method(
            self.inner(),
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
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`ScreenCastProxy::create_session`].
    /// * `identifier` - Identifier for the application window.
    ///
    /// # Return
    ///
    /// A list of [`Stream`] and an optional restore token.
    ///
    /// # Specifications
    ///
    /// See also [`Start`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-ScreenCast.Start).
    #[doc(alias = "Start")]
    pub async fn start(
        &self,
        session: &SessionProxy<'_>,
        identifier: &WindowIdentifier,
    ) -> Result<(Vec<Stream>, Option<String>), Error> {
        let options = StartCastOptions::default();
        let streams: Streams = call_request_method(
            self.inner(),
            &options.handle_token,
            "Start",
            &(session, &identifier, &options),
        )
        .await?;
        Ok((streams.streams.to_vec(), streams.restore_token))
    }

    /// Available cursor mode.
    ///
    /// # Specifications
    ///
    /// See also [`AvailableCursorModes`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-property-org-freedesktop-portal-ScreenCast.AvailableCursorModes).
    #[doc(alias = "AvailableCursorModes")]
    pub async fn available_cursor_modes(&self) -> Result<BitFlags<CursorMode>, Error> {
        self.inner()
            .get_property::<BitFlags<CursorMode>>("AvailableCursorModes")
            .await
            .map_err(From::from)
    }

    /// Available source types.
    ///
    /// # Specifications
    ///
    /// See also [`AvailableSourceTypes`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-property-org-freedesktop-portal-ScreenCast.AvailableSourceTypes).
    #[doc(alias = "AvailableSourceTypes")]
    pub async fn available_source_types(&self) -> Result<BitFlags<SourceType>, Error> {
        self.inner()
            .get_property::<BitFlags<SourceType>>("AvailableSourceTypes")
            .await
            .map_err(From::from)
    }
}

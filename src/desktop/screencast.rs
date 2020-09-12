use crate::WindowIdentifier;
use enumflags2::BitFlags;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{Fd, ObjectPath, OwnedObjectPath, Value};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Copy, Clone, Debug, Type, BitFlags)]
#[repr(u32)]
pub enum SourceType {
    Monitor = 1,
    Window = 2,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Copy, Clone, Type, BitFlags)]
#[repr(u32)]
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
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
    /// A string that will be used as the last element of the session handle.
    pub session_handle_token: Option<String>,
}

impl CreateSessionOptions {
    pub fn handle_token(mut self, handle_token: &str) -> Self {
        self.handle_token = Some(handle_token.to_string());
        self
    }

    pub fn session_handle_token(mut self, session_handle_token: &str) -> Self {
        self.session_handle_token = Some(session_handle_token.to_string());
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options on a select sources request.
pub struct SelectSourcesOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
    /// What types of content to record.
    pub types: Option<BitFlags<SourceType>>,
    /// Whether to allow selecting multiple sources.
    pub multiple: Option<bool>,
    /// Determines how the cursor will be drawn in the screen cast stream.
    pub cursor_mode: Option<BitFlags<CursorMode>>,
}

impl SelectSourcesOptions {
    pub fn handle_token(mut self, handle_token: &str) -> Self {
        self.handle_token = Some(handle_token.to_string());
        self
    }

    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = Some(multiple);
        self
    }

    pub fn cursor_mode(mut self, cursor_mode: BitFlags<CursorMode>) -> Self {
        self.cursor_mode = Some(cursor_mode);
        self
    }

    pub fn types(mut self, types: BitFlags<SourceType>) -> Self {
        self.types = Some(types);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a start screencast request.
pub struct StartCastOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
}

impl StartCastOptions {
    pub fn handle_token(mut self, handle_token: &str) -> Self {
        self.handle_token = Some(handle_token.to_string());
        self
    }
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
    /// Returns a [`Request`] handle
    ///
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn create_session(&self, options: CreateSessionOptions) -> Result<OwnedObjectPath>;

    /// Open a file descriptor to the PipeWire remote where the screen cast streams are available.
    ///
    /// Returns a file descriptor of an open PipeWire remote.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - Object path for the [`Session`] object
    /// * `options` - ?
    ///
    /// [`Session`]: ../session/struct.SessionProxy.html
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
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `session_handle` - Object path of the [`Session`] object
    /// * `options` - A `SelectSourcesOptions`
    ///
    /// [`Request`]: ../request/struct.RequestProxy.html
    /// [`Session`]: ../session/struct.SessionProxy.html
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
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `session_handle` - Object path of the [`Session`] object
    /// * `parent_window` - Identifier for the application window
    /// * `options` - A `StartScreenCastOptions`
    ///
    /// [`Request`]: ../request/struct.RequestProxy.html
    /// [`Session`]: ../session/struct.SessionProxy.html
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

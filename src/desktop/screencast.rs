use crate::WindowIdentifier;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{ObjectPath, OwnedObjectPath, Value};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
pub enum SourceType {
    Monitor = 1,
    Window = 2,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
pub enum CursorMode {
    /// The cursor is not part of the screen cast stream.
    Hidden = 1,
    /// The cursor is embedded as part of the stream buffers.
    Embedded = 2,
    /// The cursor is not part of the screen cast stream, but sent as PipeWire stream metadata.
    Metadata = 4,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options on a create a screencast session request.
pub struct CreateSessionOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
    /// A string that will be used as the last element of the session handle.
    pub session_handle_token: Option<String>,
}

impl Default for CreateSessionOptions {
    fn default() -> Self {
        Self {
            handle_token: None,
            session_handle_token: None,
        }
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options on a select sources request.
pub struct SelectSourcesOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
    /// What types of content to record.
    pub types: Option<u32>,
    /// Whether to allow selecting multiple sources.
    pub multiple: Option<bool>,
    /// Determines how the cursor will be drawn in the screen cast stream.
    pub cursor_mode: Option<u32>,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options on a start screencast request.
pub struct StartCastOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
}

impl Default for StartCastOptions {
    fn default() -> Self {
        Self { handle_token: None }
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
    ) -> Result<RawFd>;

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

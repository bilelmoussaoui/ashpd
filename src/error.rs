use crate::desktop::request::ResponseError;

#[derive(Debug)]
/// The error type for ashpd.
pub enum Error {
    /// The portal request didn't succeed.
    Portal(ResponseError),
    /// A zbus::fdo specific error.
    ZbusFdo(zbus::fdo::Error),
    /// A zbus specific error.
    Zbus(zbus::Error),
    /// Failure to parse a response's body.
    DBusMalformedMessage(zbus::MessageError),
    /// A signal returned no response.
    NoResponse,
    /// Failed to register a game with [`GameModeProxy::register_game`](`crate::desktop::game_mode::GameModeProxy::register_game`).
    RegisterGameRejected,
    /// Failed to trash a file, caused by [`TrashProxy::trash_file`](`crate::desktop::trash::TrashProxy::trash_file`).
    TrashFailed,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Portal(e) => f.write_str(&format!("Portal response error: {}", e)),
            Self::ZbusFdo(e) => f.write_str(&format!("zbus fdo error: {}", e)),
            Self::DBusMalformedMessage(e) => {
                f.write_str(&format!("zbus malformed message error: {}", e))
            }
            Self::Zbus(e) => f.write_str(&format!("zbus error: {}", e)),
            Self::NoResponse => f.write_str("portal error: no response"),
            Self::RegisterGameRejected => f.write_str("Failed to register/un-register game mode"),
            Self::TrashFailed => f.write_str("Failed to trash a file"),
        }
    }
}
impl From<ResponseError> for Error {
    fn from(e: ResponseError) -> Self {
        Self::Portal(e)
    }
}

impl From<zbus::MessageError> for Error {
    fn from(e: zbus::MessageError) -> Self {
        Self::DBusMalformedMessage(e)
    }
}

impl From<zbus::Error> for Error {
    fn from(e: zbus::Error) -> Self {
        Self::Zbus(e)
    }
}

impl From<zbus::fdo::Error> for Error {
    fn from(e: zbus::fdo::Error) -> Self {
        Self::ZbusFdo(e)
    }
}

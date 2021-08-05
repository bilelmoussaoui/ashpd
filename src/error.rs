use crate::desktop::request::ResponseError;
use zbus_macros::DBusError;

#[derive(DBusError, Debug)]
#[dbus_error(prefix = "org.freedesktop.portal.Error")]
/// An error type that describes the various DBus errors.
///
/// See https://github.com/flatpak/xdg-desktop-portal/blob/master/src/xdp-utils.h#L119-L127.
pub enum PortalError {
    ZBus(zbus::Error),
    Failed,
    InvalidArgument,
    NotFound,
    Exists,
    NotAllowed,
    Cancelled,
    WindowDestroyed,
}

#[derive(Debug)]
/// The error type for ashpd.
pub enum Error {
    /// The portal request didn't succeed.
    Response(ResponseError),
    /// Something Failed on the portal request.
    Portal(PortalError),
    /// A zbus::fdo specific error.
    Zbus(zbus::fdo::Error),
    /// A signal returned no response.
    NoResponse,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Response(e) => f.write_str(&format!("Portal request didn't succeed: {}", e)),
            Self::Zbus(e) => f.write_str(&format!("ZBus Error: {}", e)),
            Self::Portal(e) => f.write_str(&format!("Portal request failed: {}", e)),
            Self::NoResponse => f.write_str("Portal error: no response"),
        }
    }
}
impl From<ResponseError> for Error {
    fn from(e: ResponseError) -> Self {
        Self::Response(e)
    }
}

impl From<PortalError> for Error {
    fn from(e: PortalError) -> Self {
        Self::Portal(e)
    }
}

impl From<zbus::Error> for Error {
    fn from(e: zbus::Error) -> Self {
        Self::Portal(PortalError::ZBus(e))
    }
}

impl From<zbus::fdo::Error> for Error {
    fn from(e: zbus::fdo::Error) -> Self {
        Self::Zbus(e)
    }
}

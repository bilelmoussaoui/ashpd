mod activation_token;
mod app_id;
mod window_identifier;

#[cfg(feature = "backend")]
#[cfg_attr(docsrs, doc(cfg(feature = "backend")))]
pub use self::window_identifier::WindowIdentifierType;
pub use self::{
    activation_token::ActivationToken,
    app_id::{AppID, DocumentID},
    window_identifier::{MaybeWindowIdentifierExt, WindowIdentifier},
};

#[derive(Debug)]
pub enum Error {
    WaylandNoResponse,
    InvalidArgument(String),
    InvalidAppID,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WaylandNoResponse => write!(f, "Portal error: no response"),
            Self::InvalidArgument(e) => write!(f, "Invalid Argument: {e}"),
            Self::InvalidAppID => write!(f, "Invalid App ID"),
        }
    }
}

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use zvariant_derive::Type;

/// A handle token is a DBus Object Path element, specified in the [`RequestProxy`] or [`SessionProxy`]
/// object path following this format `/org/freedesktop/portal/desktop/request/SENDER/TOKEN`
/// where sender is the caller's unique name and token is the HandleToken.
///
/// A valid object path element must only contain the ASCII characters "[A-Z][a-z][0-9]_"
///
/// ```
/// use ashpd::HandleToken;
/// use std::convert::TryFrom;
///
/// assert_eq!(HandleToken::try_from("token").is_ok(), true);
///
/// assert_eq!(HandleToken::try_from("/test").is_ok(), false);
///
/// assert_eq!(HandleToken::try_from("تجربة").is_ok(), false);
/// ```
///
/// [`SessionProxy`]: ./struct.SessionProxy.html
/// [`RequestProxy`]: ./struct.RequestProxy.html
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize, Type)]
pub struct HandleToken(String);

#[derive(Debug)]
pub struct HandleInvalidCharacter(char);

impl std::fmt::Display for HandleInvalidCharacter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Invalid Character {}", self.0))
    }
}
impl std::error::Error for HandleInvalidCharacter {}

impl TryFrom<&str> for HandleToken {
    type Error = HandleInvalidCharacter;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        for char in value.chars() {
            if !char.is_ascii_alphanumeric() {
                return Err(HandleInvalidCharacter(char));
            }
        }
        Ok(Self(value.to_string()))
    }
}

impl TryFrom<String> for HandleToken {
    type Error = HandleInvalidCharacter;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        HandleToken::try_from(value.as_str())
    }
}

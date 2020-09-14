use serde::{Deserialize, Serialize};
use zvariant_derive::Type;

#[derive(Debug, Serialize, Deserialize, Type)]
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
/// [`SessionProxy`]: ../session/struct.SessionProxy.html
/// [`RequestProxy`]: ../request/struct.RequestProxy.html
pub struct HandleToken(String);

#[derive(Debug)]
pub struct HandleInvalidCharacter(char);

impl std::convert::TryFrom<&str> for HandleToken {
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

impl std::convert::TryFrom<String> for HandleToken {
    type Error = HandleInvalidCharacter;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        HandleToken::try_from(value.as_str())
    }
}

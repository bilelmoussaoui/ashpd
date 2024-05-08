use std::{
    convert::TryFrom,
    fmt::{self, Debug, Display},
    str::FromStr,
};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use zbus::{names::OwnedMemberName, zvariant::Type};

/// A handle token is a DBus Object Path element, specified in the
/// [`Request`](crate::desktop::Request)  or
/// [`Session`](crate::desktop::Session) object path following this format
/// `/org/freedesktop/portal/desktop/request/SENDER/TOKEN` where sender is the
/// caller's unique name and token is the [`HandleToken`].
///
/// A valid object path element must only contain the ASCII characters
/// `[A-Z][a-z][0-9]_`
#[derive(Serialize, Deserialize, Type)]
pub struct HandleToken(OwnedMemberName);

impl Display for HandleToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Debug for HandleToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("HandleToken")
            .field(&self.0.as_str())
            .finish()
    }
}

impl Default for HandleToken {
    fn default() -> Self {
        let mut rng = thread_rng();
        let token: String = (&mut rng)
            .sample_iter(Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        HandleToken::from_str(&format!("ashpd_{token}")).unwrap()
    }
}

#[derive(Debug)]
pub struct HandleInvalidCharacter(char);

impl std::fmt::Display for HandleInvalidCharacter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Invalid Character {}", self.0))
    }
}

impl std::error::Error for HandleInvalidCharacter {}

impl std::str::FromStr for HandleToken {
    type Err = HandleInvalidCharacter;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        for char in value.chars() {
            if !char.is_ascii_alphanumeric() && char != '_' {
                return Err(HandleInvalidCharacter(char));
            }
        }
        Ok(Self(
            OwnedMemberName::try_from(value).expect("Invalid handle token"),
        ))
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::HandleToken;

    #[test]
    fn handle_token() {
        assert!(HandleToken::from_str("token").is_ok());

        let token = HandleToken::from_str("token2").unwrap();
        assert_eq!(token.to_string(), "token2".to_string());

        assert!(HandleToken::from_str("/test").is_err());

        assert!(HandleToken::from_str("تجربة").is_err());

        assert!(HandleToken::from_str("test_token").is_ok());

        HandleToken::default(); // ensure we don't panic
    }
}

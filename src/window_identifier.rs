use serde::{Deserialize, Serialize};
use zvariant_derive::Type;

#[derive(Type, Clone, Debug, Serialize, Deserialize)]
pub struct WindowIdentifier(String);

impl WindowIdentifier {
    pub fn new(identifier: &str) -> Self {
        Self {
            0: identifier.to_string(),
        }
    }
}

impl Default for WindowIdentifier {
    fn default() -> Self {
        Self::new("")
    }
}

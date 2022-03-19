use std::fmt;

use super::WindowIdentifierType;
pub struct X11WindowIdentifier(WindowIdentifierType);

impl X11WindowIdentifier {
    pub fn new(xid: u64) -> Self {
        Self(WindowIdentifierType::X11(xid))
    }
}

impl fmt::Display for X11WindowIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}", self.0))
    }
}

use std::fmt;

pub struct X11WindowIdentifier(u64);

impl X11WindowIdentifier {
    pub fn new(xid: u64) -> Self {
        Self(xid)
    }
}

impl fmt::Display for X11WindowIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&to_handle(self.0))
    }
}

pub fn to_handle(xid: u64) -> String {
    format!("x11:0x{:x}", xid)
}

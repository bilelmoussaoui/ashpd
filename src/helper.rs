use serde::{Deserialize, Serialize};
use std::fmt;
use zvariant_derive::Type;

/// Null terminated string, we can't use CString because it doesn't implement Type.
#[derive(Serialize, Deserialize, Type)]
pub struct NString(Vec<u8>);

impl From<&str> for NString {
    fn from(t: &str) -> NString {
        let mut data: Vec<u8> = t.into();
        data.push(0);
        NString(data)
    }
}

impl From<String> for NString {
    fn from(t: String) -> NString {
        t.as_str().into()
    }
}

impl From<&NString> for String {
    fn from(t: &NString) -> Self {
        let ct = t.0.split_last().unwrap().1;
        String::from_utf8(ct.to_vec()).unwrap()
    }
}

impl fmt::Debug for NString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = String::from(self);
        write!(f, "{}", t)
    }
}

impl fmt::Display for NString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = String::from(self);
        write!(f, "{}", t)
    }
}

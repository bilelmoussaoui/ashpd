use serde::ser::{Serialize, SerializeTuple};
use zbus::zvariant::{self, Type};

/// A re-implementation of [GIcon](https://docs.gtk.org/gio/iface.Icon.html) serialization.
///
/// Portals like [`Notification`](crate::desktop::notification) and Dynamic Launcher
/// requires passing an Icon that is serialized as a `(sv)`.
#[derive(Debug)]
pub enum Icon<'a> {
    /// URI to an icon file
    Uri(&'a str),
    /// List of icon theme names
    Names(&'a [&'a str]),
    /// Icon's data
    Bytes(&'a [u8]),
}

impl<'a> Type for Icon<'a> {
    fn signature() -> zvariant::Signature<'static> {
        zvariant::Signature::from_static_str_unchecked("(sv)")
    }
}

impl<'a> Serialize for Icon<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut tuple = serializer.serialize_tuple(2)?;
        match self {
            Self::Uri(uri) => {
                tuple.serialize_element("file")?;
                tuple.serialize_element(&zvariant::Value::from(uri))?;
            }
            Self::Names(names) => {
                tuple.serialize_element("themed")?;
                let mut array = zvariant::Array::new(String::signature());
                for name in names.iter() {
                    // Safe to unwrap because we are sure it is of the correct type
                    array.append(zvariant::Value::from(name)).unwrap();
                }
                tuple.serialize_element(&zvariant::Value::Array(array))?;
            }
            Self::Bytes(bytes) => {
                tuple.serialize_element("bytes")?;
                let mut array = zvariant::Array::new(u8::signature());
                for byte in bytes.iter() {
                    // Safe to unwrap because we are sure it is of the correct type
                    array.append(zvariant::Value::U8(*byte)).unwrap();
                }
                tuple.serialize_element(&zvariant::Value::Array(array))?;
            }
        }
        tuple.end()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_icon_signature() {
        assert_eq!(Icon::signature(), "(sv)");
    }
}

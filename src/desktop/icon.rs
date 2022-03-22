use serde::ser::{Serialize, SerializeTuple};
use zbus::zvariant::{self, Type};

#[derive(Debug)]
pub enum Icon {
    Uri(String),
    Names(Vec<String>),
    Bytes(Vec<u8>),
}

impl Type for Icon {
    fn signature() -> zvariant::Signature<'static> {
        zvariant::Signature::from_static_str_unchecked("(sv)")
    }
}

impl Serialize for Icon {
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
                tuple.serialize_element(&names)?;
            }
            Self::Bytes(bytes) => {
                tuple.serialize_element("bytes")?;
                tuple.serialize_element(&zvariant::Value::from(bytes))?;
            }
        }
        tuple.end()
    }
}


#[cfg(test)]
mod test {
    use serde::Serialize;

    use super::*;

    #[test]
    fn check_icon_signature() {
        assert_eq!(Icon::signature(), "(sv)");
    }
}

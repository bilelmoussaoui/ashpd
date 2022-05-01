use serde::{
    de,
    ser::{Serialize, SerializeTuple},
    Deserialize,
};
use zbus::zvariant::{self, OwnedValue, Type};

use crate::Error;

#[derive(Debug, PartialEq, Eq, Type)]
#[zvariant(signature = "(sv)")]
pub enum Icon {
    Uri(String),
    Names(Vec<String>),
    Bytes(Vec<u8>),
}

impl Icon {
    pub fn from_names(names: &[&str]) -> Self {
        Self::Names(
            names
                .iter()
                .map(|name| name.to_owned().to_owned())
                .collect(),
        )
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
                tuple.serialize_element(&zvariant::Value::from(array))?;
            }
            Self::Bytes(bytes) => {
                tuple.serialize_element("bytes")?;
                let mut array = zvariant::Array::new(u8::signature());
                for byte in bytes.iter() {
                    // Safe to unwrap because we are sure it is of the correct type
                    array.append(zvariant::Value::from(*byte)).unwrap();
                }
                tuple.serialize_element(&zvariant::Value::from(array))?;
            }
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for Icon {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (type_, data) = <(String, OwnedValue)>::deserialize(deserializer)?;
        match type_.as_str() {
            "file" => Ok(Self::Uri(data.try_into().map_err(|_| {
                de::Error::custom("Couldn't deserialize Icon of type 'file'")
            })?)),
            "bytes" => {
                let array = data.downcast_ref::<zvariant::Array>().unwrap();
                let mut bytes = Vec::with_capacity(array.len());
                for byte in array.iter() {
                    bytes.push(*byte.downcast_ref::<u8>().unwrap());
                }
                Ok(Self::Bytes(bytes))
            }
            "themed" => {
                let array = data.downcast_ref::<zvariant::Array>().unwrap();
                let mut names = Vec::with_capacity(array.len());
                for value in array.iter() {
                    let name = value.downcast_ref::<zvariant::Str>().unwrap();
                    names.push(name.as_str().to_owned());
                }
                Ok(Self::Names(names))
            }
            _ => Err(de::Error::custom("Invalid Icon type")),
        }
    }
}

impl TryFrom<OwnedValue> for Icon {
    type Error = crate::Error;
    fn try_from(value: OwnedValue) -> Result<Self, Self::Error> {
        let structure = value.downcast_ref::<zvariant::Structure>().unwrap();
        let fields = structure.fields();
        let type_ = fields[0].downcast_ref::<zvariant::Str>().unwrap();
        match type_.as_str() {
            "file" => {
                let uri = fields[1].downcast_ref::<zvariant::Str>().unwrap();
                Ok(Self::Uri(uri.as_str().to_owned()))
            }
            "bytes" => {
                let array = fields[1].downcast_ref::<zvariant::Array>().unwrap();
                let mut bytes = Vec::with_capacity(array.len());
                for byte in array.iter() {
                    bytes.push(*byte.downcast_ref::<u8>().unwrap());
                }
                Ok(Self::Bytes(bytes))
            }
            "themed" => {
                let array = fields[1].downcast_ref::<zvariant::Array>().unwrap();
                let mut names = Vec::with_capacity(array.len());
                for value in array.iter() {
                    let name = value.downcast_ref::<zvariant::Str>().unwrap();
                    names.push(name.as_str().to_owned());
                }
                Ok(Self::Names(names))
            }
            _ => Err(Error::ParseError("Invalid Icon type")),
        }
    }
}

#[cfg(test)]
mod test {
    use byteorder::LE;
    use zbus::zvariant::{from_slice, to_bytes, EncodingContext as Context};

    use super::*;

    #[test]
    fn check_icon_signature() {
        assert_eq!(Icon::signature(), "(sv)");
    }

    #[test]
    fn serialize_deserialize() {
        let ctxt = Context::<LE>::new_dbus(0);

        let icon = Icon::from_names(&["dialog-symbolic"]);

        let encoded = to_bytes(ctxt, &icon).unwrap();
        let decoded: Icon = from_slice(&encoded, ctxt).unwrap();
        assert_eq!(decoded, icon);

        let icon = Icon::Uri("file://some/icon.png".to_owned());
        let encoded = to_bytes(ctxt, &icon).unwrap();
        let decoded: Icon = from_slice(&encoded, ctxt).unwrap();
        assert_eq!(decoded, icon);

        let icon = Icon::Bytes(vec![1, 0, 1, 0]);
        let encoded = to_bytes(ctxt, &icon).unwrap();
        let decoded: Icon = from_slice(&encoded, ctxt).unwrap();
        assert_eq!(decoded, icon);
    }
}

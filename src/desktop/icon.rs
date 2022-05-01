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
            "bytes" => Ok(Self::Bytes(data.try_into().map_err(|_| {
                de::Error::custom("Couldn't deserialize Icon of type 'bytes'")
            })?)),
            "themed" => Ok(Self::Names(data.try_into().map_err(|_| {
                de::Error::custom("Couldn't deserialize Icon of type 'themed'")
            })?)),
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
    use super::*;

    #[test]
    fn check_icon_signature() {
        assert_eq!(Icon::signature(), "(sv)");
    }

    #[test]
    fn serialize_deserialize() {
        let icon = Icon::from_names(&["dialog-info-symbolic"]);
        let string = serde_json::to_string(&icon).unwrap();
        let output: Icon = serde_json::from_str(&string).unwrap();
        assert_eq!(output, icon);
    }
}

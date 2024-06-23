use serde::{
    de,
    ser::{Serialize, SerializeTuple},
    Deserialize,
};
use zbus::zvariant::{self, OwnedValue, Structure, StructureBuilder, Type, Value};

use crate::Error;

///
#[derive(Default, Debug, PartialEq, Eq, Clone, Type)]
pub struct Pixmap {
    width: i32,
    height: i32,
    bytes: Vec<u8>,
}

impl From<Pixmap> for Structure<'_> {
    fn from(value: Pixmap) -> Self {
        StructureBuilder::new()
            .add_field(value.width)
            .add_field(value.height)
            .add_field(value.bytes)
            .build()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Type)]
#[zvariant(signature = "(sv)")]
/// A representation of an icon.
///
/// Used by both the Notification & Dynamic launcher portals.
pub enum Icon {
    /// An icon URI.
    Uri(url::Url),
    /// Name of the icon
    Name(String),
    /// A list of icon names.
    Names(Vec<String>),
    /// Icon bytes.
    Bytes(Vec<u8>),
    ///
    Pixmaps(Vec<Pixmap>),
}

impl Icon {
    /// Create an icon from a name
    pub fn with_name(name: impl Into<String>) -> Self {
        Self::Name(name.into())
    }

    /// Create an icon from a list of names.
    pub fn with_names<N>(names: impl IntoIterator<Item = N>) -> Self
    where
        N: ToString,
    {
        Self::Names(names.into_iter().map(|name| name.to_string()).collect())
    }

    pub(crate) fn is_bytes(&self) -> bool {
        matches!(self, Self::Bytes(_))
    }

    pub(crate) fn inner_bytes(&self) -> Value {
        match self {
            Self::Bytes(bytes) => {
                let mut array = zvariant::Array::new(u8::signature());
                for byte in bytes.iter() {
                    // Safe to unwrap because we are sure it is of the correct type
                    array.append(Value::from(*byte)).unwrap();
                }
                Value::from(array)
            }
            _ => panic!("Only bytes icons can be converted to a bytes variant"),
        }
    }

    pub(crate) fn as_value(&self) -> Value {
        let tuple = match self {
            Self::Uri(uri) => ("file", Value::from(uri.as_str())),
            Self::Name(name) => ("themed", Value::from(name)),
            Self::Names(names) => {
                let mut array = zvariant::Array::new(String::signature());
                for name in names.iter() {
                    // Safe to unwrap because we are sure it is of the correct type
                    array.append(Value::from(name)).unwrap();
                }

                ("themed", Value::from(array))
            }
            Self::Bytes(_) => ("bytes", self.inner_bytes()),
            Self::Pixmaps(pixmaps) => ("bytes", Value::from(pixmaps)),
        };
        Value::new(tuple)
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
                tuple.serialize_element(&Value::from(uri.as_str()))?;
            }
            Self::Names(names) => {
                tuple.serialize_element("themed")?;
                let mut array = zvariant::Array::new(String::signature());
                for name in names.iter() {
                    // Safe to unwrap because we are sure it is of the correct type
                    array.append(Value::from(name)).unwrap();
                }
                tuple.serialize_element(&Value::from(array))?;
            }
            Self::Bytes(_) => {
                tuple.serialize_element("bytes")?;
                tuple.serialize_element(&self.inner_bytes())?;
            }
            _ => {}
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
            "file" => {
                let uri_str = data.downcast_ref::<zvariant::Str>().unwrap();
                let uri = url::Url::parse(uri_str.as_str())
                    .map_err(|_| de::Error::custom("Couldn't deserialize Icon of type 'file'"))?;
                Ok(Self::Uri(uri))
            }
            "bytes" => {
                let array = data.downcast_ref::<zvariant::Array>().unwrap();
                let mut bytes = Vec::with_capacity(array.len());
                for byte in array.inner() {
                    bytes.push(byte.downcast_ref::<u8>().unwrap());
                }
                Ok(Self::Bytes(bytes))
            }
            "themed" => {
                let array = data.downcast_ref::<zvariant::Array>().unwrap();
                let mut names = Vec::with_capacity(array.len());
                for value in array.inner() {
                    let name = value.downcast_ref::<zvariant::Str>().unwrap();
                    names.push(name.as_str().to_owned());
                }
                Ok(Self::Names(names))
            }
            _ => Err(de::Error::custom("Invalid Icon type")),
        }
    }
}

impl TryFrom<&OwnedValue> for Icon {
    type Error = crate::Error;
    fn try_from(value: &OwnedValue) -> Result<Self, Self::Error> {
        let structure = value.downcast_ref::<zvariant::Structure>().unwrap();
        let fields = structure.fields();
        let type_ = fields[0].downcast_ref::<zvariant::Str>().unwrap();
        match type_.as_str() {
            "file" => {
                let uri_str = fields[1]
                    .downcast_ref::<zvariant::Str>()
                    .unwrap()
                    .to_owned();
                let uri = url::Url::parse(uri_str.as_str())
                    .map_err(|_| crate::Error::ParseError("Failed to parse uri"))?;
                Ok(Self::Uri(uri))
            }
            "bytes" => {
                let array = fields[1].downcast_ref::<zvariant::Array>().unwrap();
                let mut bytes = Vec::with_capacity(array.len());
                for byte in array.inner() {
                    bytes.push(byte.downcast_ref::<u8>().unwrap());
                }
                Ok(Self::Bytes(bytes))
            }
            "themed" => {
                let array = fields[1].downcast_ref::<zvariant::Array>().unwrap();
                let mut names = Vec::with_capacity(array.len());
                for value in array.inner() {
                    let name = value.downcast_ref::<zvariant::Str>().unwrap();
                    names.push(name.as_str().to_owned());
                }
                Ok(Self::Names(names))
            }
            _ => Err(Error::ParseError("Invalid Icon type")),
        }
    }
}

impl TryFrom<OwnedValue> for Icon {
    type Error = crate::Error;
    fn try_from(value: OwnedValue) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<Value<'_>> for Icon {
    type Error = crate::Error;
    fn try_from(value: Value<'_>) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}
impl TryFrom<&Value<'_>> for Icon {
    type Error = crate::Error;
    fn try_from(value: &Value<'_>) -> Result<Self, Self::Error> {
        Self::try_from(value.try_to_owned()?)
    }
}

#[cfg(test)]
mod test {
    use zbus::zvariant::{serialized::Context, to_bytes, Endian};

    use super::*;

    #[test]
    fn check_icon_signature() {
        assert_eq!(Icon::signature(), "(sv)");
    }

    #[test]
    fn serialize_deserialize() {
        let ctxt = Context::new_dbus(Endian::Little, 0);

        let icon = Icon::with_names(["dialog-symbolic"]);

        let encoded = to_bytes(ctxt, &icon).unwrap();
        let decoded: Icon = encoded.deserialize().unwrap().0;
        assert_eq!(decoded, icon);

        let icon = Icon::Uri(url::Url::parse("file://some/icon.png").unwrap());
        let encoded = to_bytes(ctxt, &icon).unwrap();
        let decoded: Icon = encoded.deserialize().unwrap().0;
        assert_eq!(decoded, icon);

        let icon = Icon::Bytes(vec![1, 0, 1, 0]);
        let encoded = to_bytes(ctxt, &icon).unwrap();
        let decoded: Icon = encoded.deserialize().unwrap().0;
        assert_eq!(decoded, icon);
    }
}

use std::os::fd::AsFd;

use serde::{
    Deserialize, de,
    ser::{Serialize, SerializeTuple},
};
use zbus::wire::{self, OwnedVariant, Type, Variant};

use crate::{Error, Uri};

#[derive(Debug, Type)]
#[zbus(signature = "(sv)")]
/// A representation of an icon.
///
/// Used by both the Notification & Dynamic launcher portals.
pub enum Icon {
    /// An icon URI.
    Uri(Uri),
    /// A list of icon names.
    Names(Vec<String>),
    /// Icon bytes.
    Bytes(Vec<u8>),
    /// A file descriptor.
    FileDescriptor(std::os::fd::OwnedFd),
}

impl Icon {
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

    pub(crate) fn inner_bytes<'a>(&'a self) -> Variant<'a> {
        match self {
            Self::Bytes(bytes) => {
                let mut array = wire::Array::new(u8::SIGNATURE);
                for byte in bytes.iter() {
                    // Safe to unwrap because we are sure it is of the correct type
                    array.append(Variant::from(*byte)).unwrap();
                }
                Variant::from(array)
            }
            _ => panic!("Only bytes icons can be converted to a bytes variant"),
        }
    }

    pub(crate) fn as_variant<'a>(&'a self) -> Variant<'a> {
        let tuple = match self {
            Self::Uri(uri) => ("file", Variant::from(uri.as_str())),
            Self::Names(names) => {
                let mut array = wire::Array::new(String::SIGNATURE);
                for name in names.iter() {
                    // Safe to unwrap because we are sure it is of the correct type
                    array.append(Variant::from(name)).unwrap();
                }

                ("themed", Variant::from(array))
            }
            Self::Bytes(_) => ("bytes", self.inner_bytes()),
            Self::FileDescriptor(fd) => ("file-descriptor", wire::Fd::from(fd).into()),
        };
        Variant::new(tuple)
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
                tuple.serialize_element(&Variant::from(uri.as_str()))?;
            }
            Self::Names(names) => {
                tuple.serialize_element("themed")?;
                let mut array = wire::Array::new(String::SIGNATURE);
                for name in names.iter() {
                    // Safe to unwrap because we are sure it is of the correct type
                    array.append(Variant::from(name)).unwrap();
                }
                tuple.serialize_element(&Variant::from(array))?;
            }
            Self::Bytes(_) => {
                tuple.serialize_element("bytes")?;
                tuple.serialize_element(&self.inner_bytes())?;
            }
            Self::FileDescriptor(fd) => {
                tuple.serialize_element("file-descriptor")?;
                tuple.serialize_element(&Variant::from(wire::Fd::from(fd)))?;
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
        let (type_, data) = <(String, OwnedVariant)>::deserialize(deserializer)?;
        match type_.as_str() {
            "file" => {
                let uri_str = data.downcast_ref::<wire::Str>().unwrap();
                let uri = Uri::parse(uri_str.as_str())
                    .map_err(|e| serde::de::Error::custom(format!("Invalid URI: {}", e)))?;
                Ok(Self::Uri(uri))
            }
            "bytes" => {
                let array = data.downcast_ref::<wire::Array>().unwrap();
                let mut bytes = Vec::with_capacity(array.len());
                for byte in array.inner() {
                    bytes.push(byte.downcast_ref::<u8>().unwrap());
                }
                Ok(Self::Bytes(bytes))
            }
            "themed" => {
                let array = data.downcast_ref::<wire::Array>().unwrap();
                let mut names = Vec::with_capacity(array.len());
                for value in array.inner() {
                    let name = value.downcast_ref::<wire::Str>().unwrap();
                    names.push(name.as_str().to_owned());
                }
                Ok(Self::Names(names))
            }
            "file-descriptor" => {
                let fd = data.downcast_ref::<wire::Fd>().unwrap();
                Ok(Self::FileDescriptor(
                    fd.as_fd()
                        .try_clone_to_owned()
                        .expect("Failed to clone file descriptor"),
                ))
            }
            _ => Err(de::Error::custom("Invalid Icon type")),
        }
    }
}

impl TryFrom<&OwnedVariant> for Icon {
    type Error = crate::Error;
    fn try_from(value: &OwnedVariant) -> Result<Self, Self::Error> {
        let structure = value.downcast_ref::<wire::Structure>().unwrap();
        let fields = structure.fields();
        let type_ = fields[0].downcast_ref::<wire::Str>().unwrap();
        match type_.as_str() {
            "file" => {
                let uri_str = fields[1].downcast_ref::<wire::Str>().unwrap().to_owned();
                let uri = Uri::parse(uri_str.as_str()).map_err(crate::Error::Uri)?;
                Ok(Self::Uri(uri))
            }
            "bytes" => {
                let array = fields[1].downcast_ref::<wire::Array>().unwrap();
                let mut bytes = Vec::with_capacity(array.len());
                for byte in array.inner() {
                    bytes.push(byte.downcast_ref::<u8>().unwrap());
                }
                Ok(Self::Bytes(bytes))
            }
            "themed" => {
                let array = fields[1].downcast_ref::<wire::Array>().unwrap();
                let mut names = Vec::with_capacity(array.len());
                for value in array.inner() {
                    let name = value.downcast_ref::<wire::Str>().unwrap();
                    names.push(name.as_str().to_owned());
                }
                Ok(Self::Names(names))
            }
            "file-descriptor" => {
                let fd = fields[1].downcast_ref::<wire::Fd>().unwrap();
                Ok(Self::FileDescriptor(
                    fd.as_fd()
                        .try_clone_to_owned()
                        .expect("Failed to clone file descriptor"),
                ))
            }
            _ => Err(Error::ParseError("Invalid Icon type")),
        }
    }
}

impl TryFrom<OwnedVariant> for Icon {
    type Error = crate::Error;
    fn try_from(value: OwnedVariant) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<Variant<'_>> for Icon {
    type Error = crate::Error;
    fn try_from(value: Variant<'_>) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}
impl TryFrom<&Variant<'_>> for Icon {
    type Error = crate::Error;
    fn try_from(value: &Variant<'_>) -> Result<Self, Self::Error> {
        Self::try_from(value.try_to_owned()?)
    }
}

#[cfg(test)]
mod test {
    use zbus::wire::{Endian, serialized::Context, to_bytes};

    use super::*;

    #[test]
    fn check_icon_signature() {
        assert_eq!(Icon::SIGNATURE, "(sv)");
    }

    #[test]
    fn serialize_deserialize() {
        let ctxt = Context::new(Endian::Little, 0);

        let icon = Icon::with_names(["dialog-symbolic"]);

        let encoded = to_bytes(ctxt, &icon).unwrap();
        let decoded: Icon = encoded.deserialize().unwrap().0;
        assert!(matches!(decoded, Icon::Names(_)));

        let icon = Icon::Uri(Uri::parse("file://some/icon.png").unwrap());
        let encoded = to_bytes(ctxt, &icon).unwrap();
        let decoded: Icon = encoded.deserialize().unwrap().0;
        assert!(matches!(decoded, Icon::Uri(_)));

        let icon = Icon::Bytes(vec![1, 0, 1, 0]);
        let encoded = to_bytes(ctxt, &icon).unwrap();
        let decoded: Icon = encoded.deserialize().unwrap().0;
        assert!(matches!(decoded, Icon::Bytes(_)));

        let fd = std::fs::File::open("/tmp").unwrap();
        let icon = Icon::FileDescriptor(fd.as_fd().try_clone_to_owned().unwrap());
        let encoded = to_bytes(ctxt, &icon).unwrap();
        let decoded: Icon = encoded.deserialize().unwrap().0;
        assert!(matches!(decoded, Icon::FileDescriptor(_)));
    }
}

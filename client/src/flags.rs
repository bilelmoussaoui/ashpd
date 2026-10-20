use enumflags2::{BitFlag, BitFlags};

pub(crate) fn from_bits<F: BitFlag<Numeric = u32>>(bits: u32) -> zbus::Result<BitFlags<F>> {
    BitFlags::from_bits(bits).map_err(|_| zbus::Error::IncorrectType)
}

#[cfg(feature = "input_capture")]
pub(crate) fn serialize<F, S>(flags: &BitFlags<F>, serializer: S) -> Result<S::Ok, S::Error>
where
    F: BitFlag<Numeric = u32>,
    S: serde::Serializer,
{
    zbus::as_variant::serialize(&flags.bits(), serializer)
}

#[cfg(any(feature = "input_capture", feature = "remote_desktop"))]
pub(crate) fn deserialize<'de, F, D>(deserializer: D) -> Result<BitFlags<F>, D::Error>
where
    F: BitFlag<Numeric = u32>,
    D: serde::Deserializer<'de>,
{
    let bits = zbus::as_variant::deserialize(deserializer)?;
    from_bits(bits).map_err(serde::de::Error::custom)
}

#[cfg(any(
    feature = "flatpak",
    feature = "remote_desktop",
    feature = "screencast"
))]
pub(crate) mod optional {
    use super::*;

    pub(crate) fn serialize<F, S>(
        flags: &Option<BitFlags<F>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        F: BitFlag<Numeric = u32>,
        S: serde::Serializer,
    {
        zbus::as_variant::optional::serialize(&flags.map(|flags| flags.bits()), serializer)
    }

    #[cfg(feature = "screencast")]
    pub(crate) fn deserialize<'de, F, D>(deserializer: D) -> Result<Option<BitFlags<F>>, D::Error>
    where
        F: BitFlag<Numeric = u32>,
        D: serde::Deserializer<'de>,
    {
        let bits: Option<u32> = zbus::as_variant::optional::deserialize(deserializer)?;
        bits.map(from_bits)
            .transpose()
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(all(
    test,
    feature = "input_capture",
    feature = "screencast",
    feature = "backend"
))]
mod tests {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use zbus::{
        OwnedVariant, Type,
        wire::{Endian, serialized::Context, to_bytes},
    };

    use super::*;

    #[enumflags2::bitflags]
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Flag {
        First = 1,
        Second = 4,
    }

    #[derive(Debug, Serialize, Deserialize, Type)]
    #[zbus(signature = "dict")]
    struct Options {
        #[serde(with = "super")]
        flags: BitFlags<Flag>,
        #[serde(
            default,
            with = "super::optional",
            skip_serializing_if = "Option::is_none"
        )]
        optional_flags: Option<BitFlags<Flag>>,
    }

    #[test]
    fn flag_dictionaries_encode_uint32_variants() {
        let options = Options {
            flags: Flag::First | Flag::Second,
            optional_flags: Some(Flag::Second.into()),
        };
        let bytes = to_bytes(Context::new(Endian::Little, 0), &options).unwrap();
        let (values, _) = bytes
            .deserialize::<HashMap<String, OwnedVariant>>()
            .unwrap();
        assert_eq!(u32::try_from(&values["flags"]).unwrap(), 5);
        assert_eq!(u32::try_from(&values["optional_flags"]).unwrap(), 4);
        let (decoded, _) = bytes.deserialize::<Options>().unwrap();
        assert_eq!(decoded.flags, options.flags);
        assert_eq!(decoded.optional_flags, options.optional_flags);
    }

    #[test]
    fn absent_optional_flags_are_omitted_from_dictionary() {
        let options = Options {
            flags: BitFlags::empty(),
            optional_flags: None,
        };
        let bytes = to_bytes(Context::new(Endian::Little, 0), &options).unwrap();
        let (values, _) = bytes
            .deserialize::<HashMap<String, OwnedVariant>>()
            .unwrap();
        assert!(!values.contains_key("optional_flags"));
        let (decoded, _) = bytes.deserialize::<Options>().unwrap();
        assert_eq!(decoded.optional_flags, None);
    }

    #[test]
    fn flag_dictionaries_reject_unknown_bits() {
        for (flags, optional_flags) in [(2_u32, None), (1, Some(2_u32))] {
            let mut values = HashMap::from([("flags", OwnedVariant::from(flags))]);
            if let Some(bits) = optional_flags {
                values.insert("optional_flags", OwnedVariant::from(bits));
            }
            let bytes = to_bytes(Context::new(Endian::Little, 0), &values).unwrap();
            assert!(bytes.deserialize::<Options>().is_err());
        }
    }
}

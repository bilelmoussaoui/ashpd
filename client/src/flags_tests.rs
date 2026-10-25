use serde::{Serialize, de::DeserializeOwned};
use zbus::{
    Type,
    wire::{LE, serialized::Context, to_bytes},
};

fn check_flags<F>(values: &[(F, u32)])
where
    F: bitflags::Flags<Bits = u32>
        + Type
        + Serialize
        + DeserializeOwned
        + std::fmt::Debug
        + PartialEq,
{
    assert_eq!(F::SIGNATURE, u32::SIGNATURE);
    let context = Context::new(LE, 0);
    let mut combined = 0;
    for (flag, expected) in values {
        assert_eq!(flag.bits(), *expected);
        combined |= expected;
    }
    // Include empty sets, combinations, and a bit reserved for future versions.
    for bits in [0, combined, combined | (1 << 31)] {
        let flags = F::from_bits_retain(bits);
        let encoded = to_bytes(context, &flags).unwrap();
        assert_eq!(encoded.deserialize::<u32>().unwrap().0, bits);
        assert_eq!(encoded.deserialize::<F>().unwrap().0, flags);
        // Human-readable serializers must also use the integer wire value.
        assert_eq!(serde_json::to_value(&flags).unwrap(), bits);
    }
}

macro_rules! check {
    ($type:path, $($flag:ident = $bits:expr),+ $(,)?) => {{
        use $type as Flags;
        check_flags(&[$((Flags::$flag, $bits)),+]);
    }};
}

#[test]
fn portal_flags_use_unsigned_integers() {
    #[cfg(feature = "screencast")]
    {
        check!(
            crate::desktop::screencast::SourceType,
            MONITOR = 1,
            WINDOW = 2,
            VIRTUAL = 4
        );
        check!(
            crate::desktop::screencast::CursorMode,
            HIDDEN = 1,
            EMBEDDED = 2,
            METADATA = 4
        );
    }
    #[cfg(feature = "screenshot")]
    check!(
        crate::desktop::screenshot::AvailableTargets,
        SCREEN = 1,
        WINDOW = 2,
        AREA = 4,
        ACTIVE_WINDOW = 8
    );
    #[cfg(feature = "remote_desktop")]
    check!(
        crate::desktop::remote_desktop::DeviceType,
        KEYBOARD = 1,
        POINTER = 2,
        TOUCHSCREEN = 4
    );
    #[cfg(feature = "input_capture")]
    check!(
        crate::desktop::input_capture::Capabilities,
        KEYBOARD = 1,
        POINTER = 2,
        TOUCHSCREEN = 4
    );
    #[cfg(feature = "dynamic_launcher")]
    {
        check!(
            crate::desktop::dynamic_launcher::LauncherType,
            APPLICATION = 1,
            WEB_APPLICATION = 2
        );
        assert_eq!(
            crate::desktop::dynamic_launcher::LauncherType::default(),
            crate::desktop::dynamic_launcher::LauncherType::APPLICATION
        );
    }
    #[cfg(feature = "inhibit")]
    check!(
        crate::desktop::inhibit::InhibitFlags,
        LOGOUT = 1,
        USER_SWITCH = 2,
        SUSPEND = 4,
        IDLE = 8
    );
    #[cfg(feature = "documents")]
    check!(
        crate::documents::DocumentFlags,
        REUSE_EXISTING = 1,
        PERSISTENT = 2,
        AS_NEEDED_BY_APP = 4,
        EXPORT_DIRECTORY = 8
    );
    #[cfg(feature = "flatpak")]
    {
        check!(
            crate::flatpak::SandboxFlags,
            DISPLAY_ACCESS = 1,
            SOUND_ACCESS = 2,
            GPU_ACCESS = 4,
            SESSION_BUS_ACCESS = 8,
            ACCESSIBILITY_BUS_ACCESS = 16
        );
        check!(
            crate::flatpak::SpawnFlags,
            CLEAR_ENV = 1,
            LATEST_VERSION = 2,
            SANDBOX = 4,
            NO_NETWORK = 8,
            WATCH_BUS = 16,
            EXPOSE_PIDS = 32,
            NOTIFY_START = 64,
            SHARE_PIDS = 128,
            EMPTY_APP = 256
        );
        check!(crate::flatpak::SupportsFlags, EXPOSE_PIDS = 1);
        check!(
            crate::flatpak::HostCommandFlags,
            CLEAR_ENV = 1,
            WATCH_BUS = 2
        );
    }
    #[cfg(all(feature = "backend", feature = "background"))]
    check!(
        crate::backend::background::AutoStartFlags,
        DBUS_ACTIVATION = 1
    );
}

#[cfg(feature = "screencast")]
#[test]
fn optional_flags_are_variants_and_absent_fields_are_omitted() {
    use crate::desktop::screencast::{SelectSourcesOptions, SourceType};
    use std::collections::HashMap;
    use zbus::OwnedVariant;

    let context = Context::new(LE, 0);
    for sources in [
        None,
        Some(SourceType::empty()),
        Some(SourceType::MONITOR | SourceType::WINDOW),
    ] {
        let options = SelectSourcesOptions::default().set_sources(sources);
        let encoded = to_bytes(context, &options).unwrap();
        let dict: HashMap<String, OwnedVariant> = encoded.deserialize().unwrap().0;
        assert_eq!(
            dict.get("types").map(|value| u32::try_from(value).unwrap()),
            sources.map(|flags| flags.bits())
        );
        let decoded: SelectSourcesOptions = encoded.deserialize().unwrap().0;
        assert_eq!(decoded.sources(), sources);
    }
}

#[cfg(feature = "input_capture")]
#[test]
fn required_flags_are_variants_and_retain_unknown_bits() {
    use crate::desktop::input_capture::{Capabilities, CreateSessionOptions, StartResponse};
    use std::collections::HashMap;
    use zbus::OwnedVariant;

    let bits = Capabilities::KEYBOARD.bits() | Capabilities::POINTER.bits() | (1 << 31);
    let options =
        CreateSessionOptions::default().set_capabilities(Capabilities::from_bits_retain(bits));
    let context = Context::new(LE, 0);
    let encoded = to_bytes(context, &options).unwrap();
    let dict: HashMap<String, OwnedVariant> = encoded.deserialize().unwrap().0;
    assert_eq!(u32::try_from(&dict["capabilities"]).unwrap(), bits);

    let response = HashMap::from([("capabilities", OwnedVariant::from(bits))]);
    let encoded = to_bytes(context, &response).unwrap();
    let decoded: StartResponse = encoded.deserialize().unwrap().0;
    assert_eq!(decoded.capabilities().bits(), bits);
}

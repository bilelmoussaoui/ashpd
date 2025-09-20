use serde::{de::Deserializer, Deserialize};
use zbus::zvariant::Type;

use crate::{AppID, WindowIdentifierType};

pub type Result<T> = std::result::Result<T, crate::error::PortalError>;

#[derive(Debug, Default, Type)]
#[zvariant(signature = "s")]
pub(crate) struct MaybeWindowIdentifier(Option<WindowIdentifierType>);

impl MaybeWindowIdentifier {
    pub fn inner(self) -> Option<WindowIdentifierType> {
        self.0
    }
}

impl<'de> Deserialize<'de> for MaybeWindowIdentifier {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = String::deserialize(deserializer)?;
        if inner.is_empty() {
            Ok(Self(None))
        } else {
            Ok(Self(inner.parse::<WindowIdentifierType>().ok()))
        }
    }
}

#[derive(Debug, Default, Type)]
#[zvariant(signature = "s")]
pub(crate) struct MaybeAppID(Option<AppID>);

impl MaybeAppID {
    pub fn inner(self) -> Option<AppID> {
        self.0
    }
}

impl<'de> Deserialize<'de> for MaybeAppID {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = String::deserialize(deserializer)?;
        if inner.is_empty() {
            Ok(Self(None))
        } else {
            Ok(Self(inner.parse::<AppID>().ok()))
        }
    }
}

pub mod access;
pub mod account;
pub mod app_chooser;
pub mod background;
mod builder;
pub use builder::Builder;
pub mod email;
pub mod file_chooser;
pub mod lockdown;
pub mod permission_store;
pub mod print;
pub mod request;
pub mod screencast;
pub mod screenshot;
pub mod secret;
pub mod session;
pub mod settings;
mod spawn;
pub mod usb;
pub mod wallpaper;

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        fs,
        path::{Path, PathBuf},
    };

    use quick_xml::{events::Event, Reader};

    // Helper to convert PascalCase to snake_case
    fn pascal_to_snake_case(s: &str) -> String {
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_ascii_uppercase() {
                if i > 0 {
                    result.push('_');
                }
                result.push(c.to_ascii_lowercase());
            } else {
                result.push(c);
            }
        }
        result
    }

    fn extract_names_from_xml(xml_content: &str) -> HashMap<String, Vec<String>> {
        let mut interfaces = HashMap::new();
        let mut reader = Reader::from_str(xml_content);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        let mut current_interface_name = String::new();
        let mut current_names = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    // Handle both Start and Empty events
                    if let Some(Ok(attr)) = e
                        .attributes()
                        .find(|a| a.as_ref().map_or(false, |a| a.key.as_ref() == b"name"))
                    {
                        if let Ok(value) = attr.decode_and_unescape_value(reader.decoder()) {
                            match e.name().as_ref() {
                                b"interface" => {
                                    current_interface_name = value.to_string();
                                    current_names.clear();
                                }
                                b"method" | b"property" | b"signal" => {
                                    if value != "version" {
                                        current_names.push(value.to_string());
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    if e.name().as_ref() == b"interface" {
                        interfaces.insert(current_interface_name.clone(), current_names.clone());
                    }
                }
                _ => (),
            }
            buf.clear();
        }
        interfaces
    }

    #[test]
    fn test_all_backend_interfaces_have_doc_aliases() {
        let interfaces_dir = PathBuf::from("./interfaces/backend");
        assert!(
            interfaces_dir.exists(),
            "Interfaces directory not found at {}",
            interfaces_dir.display()
        );

        // Define explicit mappings for backend interfaces if needed
        let rust_file_mappings: HashMap<&str, &str> =
            HashMap::from([("org.freedesktop.impl.portal.ScreenCast", "screencast.rs")]);

        let entries = fs::read_dir(&interfaces_dir).expect("Failed to read interfaces directory");

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "xml") {
                println!("Checking XML file: {}", path.display());

                let xml_content = fs::read_to_string(&path).unwrap();
                let interfaces = extract_names_from_xml(&xml_content);

                for (interface_name, names_to_check) in interfaces {
                    // Map the D-Bus interface name to the corresponding Rust file path
                    let interface_name_pascal = interface_name
                        .strip_prefix("org.freedesktop.impl.portal.")
                        .expect("Interface name does not have the expected prefix.");

                    const IGNORED_PORTALS: &[&str; 7] = &[
                        "Clipboard",
                        "DynamicLauncher",
                        "GlobalShortcuts",
                        "Inhibit",
                        "InputCapture",
                        "Notification",
                        "RemoteDesktop",
                    ];

                    if IGNORED_PORTALS.contains(&interface_name_pascal) {
                        continue;
                    }

                    let rust_path = if let Some(mapped_path) =
                        rust_file_mappings.get(interface_name.as_str())
                    {
                        PathBuf::from(format!("src/backend/{}", mapped_path))
                    } else {
                        let rust_file_name_snake = pascal_to_snake_case(interface_name_pascal);
                        PathBuf::from(format!("src/backend/{}.rs", rust_file_name_snake))
                    };

                    // Check if the Rust file exists
                    assert!(
                        rust_path.exists(),
                        "Corresponding Rust file not found for interface '{}' at {}",
                        interface_name,
                        rust_path.display()
                    );

                    // Read the Rust file content
                    let rust_content = fs::read_to_string(&rust_path).unwrap();

                    // Assert that each name has a corresponding doc alias
                    for name in &names_to_check {
                        let alias_str = format!("#[doc(alias = \"{}\")]", name);
                        assert!(
                            rust_content.contains(&alias_str),
                            "Missing doc alias '{}' for interface '{}' in file {}",
                            alias_str,
                            interface_name,
                            rust_path.display()
                        );
                    }
                }
            }
        }
    }
}

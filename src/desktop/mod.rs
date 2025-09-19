mod handle_token;
pub(crate) mod request;
mod session;
#[cfg_attr(docsrs, doc(cfg(feature = "backend")))]
#[cfg(feature = "backend")]
pub use self::handle_token::HandleToken;
#[cfg(not(feature = "backend"))]
pub(crate) use self::handle_token::HandleToken;
pub use self::{
    request::{Request, Response, ResponseError, ResponseType},
    session::{Session, SessionPortal},
};
mod color;
pub use color::Color;
mod icon;
pub use icon::Icon;

pub mod account;
pub mod background;
pub mod camera;
pub mod clipboard;
#[deprecated = "The portal does not serve any purpose as nothing really can make use of it as is."]
pub mod device;
pub mod dynamic_launcher;
pub mod email;
/// Open/save file(s) chooser.
pub mod file_chooser;
/// Enable/disable/query the status of Game Mode.
pub mod game_mode;
/// Register global shortcuts
pub mod global_shortcuts;
/// Inhibit the session from being restarted or the user from logging out.
pub mod inhibit;
/// Capture input events from physical or logical devices.
pub mod input_capture;
/// Query the user's GPS location.
pub mod location;
/// Monitor memory level.
pub mod memory_monitor;
/// Check the status of the network on a user's machine.
pub mod network_monitor;
/// Send/withdraw notifications.
pub mod notification;
pub mod open_uri;
/// Power profile monitoring.
pub mod power_profile_monitor;
/// Print a document.
pub mod print;
/// Proxy information.
pub mod proxy_resolver;
pub mod realtime;
/// Start a remote desktop session and interact with it.
pub mod remote_desktop;
pub mod screencast;
pub mod screenshot;
/// Retrieve a per-application secret used to encrypt confidential data inside
/// the sandbox.
pub mod secret;
/// Read & listen to system settings changes.
pub mod settings;
pub mod trash;
pub mod usb;
pub mod wallpaper;

#[cfg_attr(feature = "glib", derive(glib::Enum))]
#[cfg_attr(feature = "glib", enum_type(name = "AshpdPersistMode"))]
#[derive(
    Default,
    serde_repr::Deserialize_repr,
    serde_repr::Serialize_repr,
    PartialEq,
    Eq,
    Debug,
    Copy,
    Clone,
    zbus::zvariant::Type,
)]
#[doc(alias = "XdpPersistMode")]
#[repr(u32)]
/// Persistence mode for a screencast or remote desktop session.
pub enum PersistMode {
    #[doc(alias = "XDP_PERSIST_MODE_NONE")]
    #[default]
    /// Do not persist.
    DoNot = 0,
    #[doc(alias = "XDP_PERSIST_MODE_TRANSIENT")]
    /// Persist while the application is running.
    Application = 1,
    #[doc(alias = "XDP_PERSIST_MODE_PERSISTENT")]
    /// Persist until explicitly revoked.
    ExplicitlyRevoked = 2,
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs, path::PathBuf};

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

    #[test]
    fn all_interfaces_have_implementations() {
        let interfaces_dir = PathBuf::from("./interfaces");
        assert!(
            interfaces_dir.exists(),
            "Interfaces directory not found at {}",
            interfaces_dir.display()
        );

        // Define the explicit mapping from interface name to file path
        let rust_file_mappings: HashMap<&str, &str> = HashMap::from([
            ("org.freedesktop.portal.ScreenCast", "desktop/screencast.rs"),
            ("org.freedesktop.portal.OpenURI", "desktop/open_uri.rs"),
            (
                "org.freedesktop.portal.FileTransfer",
                "documents/file_transfer.rs",
            ),
            ("org.freedesktop.portal.Documents", "documents/mod.rs"),
            ("org.freedesktop.portal.Flatpak", "flatpak/mod.rs"),
            (
                "org.freedesktop.portal.Flatpak.UpdateMonitor",
                "flatpak/update_monitor.rs",
            ),
        ]);

        let entries = fs::read_dir(&interfaces_dir).expect("Failed to read interfaces directory");

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "xml") {
                println!("Checking XML file: {}", path.display());

                let xml_content = fs::read_to_string(&path).unwrap();
                let mut reader = Reader::from_str(&xml_content);
                reader.config_mut().trim_text(true);

                let mut buf = Vec::new();
                let mut interface_name = String::new();
                let mut names_to_check = Vec::new();

                loop {
                    match reader.read_event_into(&mut buf) {
                        Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                        Ok(Event::Eof) => break,
                        Ok(Event::Start(e)) => match e.name().as_ref() {
                            b"interface" => {
                                if let Some(name_attr) = e.attributes().find_map(|a| {
                                    a.ok().filter(|attr| attr.key.as_ref() == b"name")
                                }) {
                                    if let Ok(name_value) =
                                        str::from_utf8(&name_attr.value).map(ToOwned::to_owned)
                                    {
                                        interface_name = name_value;
                                        names_to_check.clear();
                                    }
                                }
                            }
                            b"method" | b"property" | b"signal" => {
                                if let Some(name_attr) = e.attributes().find_map(|a| {
                                    a.ok().filter(|attr| attr.key.as_ref() == b"name")
                                }) {
                                    if let Ok(name_value) =
                                        str::from_utf8(&name_attr.value).map(ToOwned::to_owned)
                                    {
                                        if e.name().as_ref() == b"property"
                                            && name_value == "version"
                                        {
                                            continue;
                                        }
                                        names_to_check.push(name_value);
                                    }
                                }
                            }
                            _ => (),
                        },
                        Ok(Event::End(e)) => {
                            if e.name().as_ref() == b"interface" {
                                // Process the collected names for the interface
                                // Map the D-Bus interface name to the corresponding Rust file path
                                let rust_path = if let Some(mapped_path) =
                                    rust_file_mappings.get(interface_name.as_str())
                                {
                                    PathBuf::from(format!("src/{}", mapped_path))
                                } else {
                                    let interface_name_pascal = interface_name
                                        .strip_prefix("org.freedesktop.portal.")
                                        .expect(
                                            "Interface name does not have the expected prefix.",
                                        );
                                    let rust_file_name_snake =
                                        pascal_to_snake_case(interface_name_pascal);
                                    PathBuf::from(format!(
                                        "src/desktop/{}.rs",
                                        rust_file_name_snake
                                    ))
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
                        _ => (),
                    }
                    buf.clear();
                }
            }
        }
    }
}

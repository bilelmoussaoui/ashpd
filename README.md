# ASHPD

[![](https://docs.rs/ashpd/badge.svg)](https://docs.rs/ashpd/) [![](https://img.shields.io/crates/v/ashpd)](https://crates.io/crates/ashpd) ![](https://github.com/bilelmoussaoui/ashpd/workflows/CI/badge.svg)

ASHPD, acronym of Aperture Science Handheld Portal Device is a Rust & [zbus](https://gitlab.freedesktop.org/zeenix/zbus) wrapper of
the XDG portals DBus interfaces. The library aims to provide an easy way to
interact with the various portals defined per the [specifications](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html).
It provides an alternative to the C library [https://github.com/flatpak/libportal](https://github.com/flatpak/libportal)


```rust
use ashpd::desktop::screenshot::{Color, PickColorOptions, ScreenshotProxy};
use ashpd::{RequestProxy, Response, WindowIdentifier};
use zbus::fdo::Result;

fn main() -> Result<()> {
   let connection = zbus::Connection::new_session()?;
   let proxy = ScreenshotProxy::new(&connection)?;
   
   let request_handle = proxy.pick_color(
            WindowIdentifier::default(),
            PickColorOptions::default()
   )?;
   
   let request = RequestProxy::new(&connection, &request_handle)?;
    request.on_response(|response: Response<Color>| {
        if let Ok(color) = response {
            println!("({}, {}, {})", color.red(), color.green(), color.blue());
        }
   })?;
   
   Ok(())
 }
```

## Optional features
| Feature | Description |
| ---     | ----------- |
| feature_gtk | Implement `Into<gdk::RGBA>` for [`Color`] |
|  | Implement `From<gtk::Window>` for [`WindowIdentifier`] |

[`Color`]: https://bilelmoussaoui.github.io/ashpd/ashpd/desktop/screenshot/struct.Color.html
[`WindowIdentifier`]: https://bilelmoussaoui.github.io/ashpd/ashpd/struct.WindowIdentifier.html

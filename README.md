# ASHPD

[![docs](https://docs.rs/ashpd/badge.svg)](https://docs.rs/ashpd/) [![crates.io](https://img.shields.io/crates/v/ashpd)](https://crates.io/crates/ashpd) ![CI](https://github.com/bilelmoussaoui/ashpd/workflows/CI/badge.svg)

ASHPD, acronym of Aperture Science Handheld Portal Device is a Rust & [zbus](https://gitlab.freedesktop.org/zeenix/zbus) wrapper of
the XDG portals DBus interfaces. The library aims to provide an easy way to
interact with the various portals defined per the [specifications](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html).
It provides an alternative to the C library [https://github.com/flatpak/libportal](https://github.com/flatpak/libportal)

```rust,no_run
use ashpd::{desktop::screenshot, Response, WindowIdentifier};
use zbus::fdo::Result;

async fn run() -> Result<()> {
    let identifier = WindowIdentifier::default();
    if let Ok(Response::Ok(color)) = screenshot::pick_color(identifier).await {
        println!("({}, {}, {})", color.red(), color.green(), color.blue());
    }
    Ok(())
}
```

```rust,no_run
use ashpd::{desktop::camera, Response};
use zbus::fdo::Result;
async fn run() -> Result<()> {
    if camera::is_present().await? {
        if let Ok(Response::Ok(pipewire_fd)) = camera::stream().await {
            // Use the PipeWire file descriptor with GStreamer for example
        }
    }
    Ok(())
}
```

## Optional features

| Feature | Description |
| ---     | ----------- |
| feature_gtk3 | Implement `From<Color>` for [`gdk3::RGBA`] |
|  | Implement `From<gtk3::Window>` for [`WindowIdentifier`] |
| feature_gtk4 | Implement `From<Color>` for [`gdk4::RGBA`] |
|  | Provides ['WindowIdentifier::from_window] |

[`Color`]: https://bilelmoussaoui.github.io/ashpd/ashpd/desktop/screenshot/struct.Color.html
[`WindowIdentifier`]: https://bilelmoussaoui.github.io/ashpd/ashpd/struct.WindowIdentifier.html

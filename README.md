# ASHPD

[![docs](https://docs.rs/ashpd/badge.svg)](https://docs.rs/ashpd/) [![crates.io](https://img.shields.io/crates/v/ashpd)](https://crates.io/crates/ashpd) ![CI](https://github.com/bilelmoussaoui/ashpd/workflows/CI/badge.svg)

ASHPD, acronym of Aperture Science Handheld Portal Device is a Rust & [zbus](https://gitlab.freedesktop.org/zeenix/zbus) wrapper of
the XDG portals DBus interfaces. The library aims to provide an easy way to
interact with the various portals defined per the [specifications](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html).
It provides an alternative to the C library [https://github.com/flatpak/libportal](https://github.com/flatpak/libportal)

## Examples

Ask the compositor to pick a color

```rust,no_run
use ashpd::desktop::screenshot::ScreenshotProxy;

async fn run() -> Result<(), ashpd::Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = ScreenshotProxy::new(&connection).await?;

    let color = proxy.pick_color(Default::default).await?;
    println!("({}, {}, {})", color.red(), color.green(), color.blue());
    Ok(())
}
```

Start a PipeWire stream from the user's camera

```rust,no_run
use ashpd::desktop::camera::CameraProxy;

pub async fn run() -> Result<(), ashpd::Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = CameraProxy::new(&connection).await?;

    if proxy.is_camera_present().await? {
        proxy.access_camera().await?;
        let remote_fd = proxy.open_pipe_wire_remote().await?;
        // pass the remote fd to GStreamer for example
    }
    Ok(())
}
```

## Optional features

| Feature | Description |
| ---     | ----------- |
| feature_gtk3 | Implement `From<Color>` for `gdk3::RGBA` |
|  | Implement `From<gtk3::Window>` for `WindowIdentifier` |
| feature_gtk4 | Implement `From<Color>` for `gdk4::RGBA` |
|  | Provides `WindowIdentifier::from_window` |

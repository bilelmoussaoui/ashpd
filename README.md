# ASHPD

[![docs](https://docs.rs/ashpd/badge.svg)](https://docs.rs/ashpd/) [![crates.io](https://img.shields.io/crates/v/ashpd)](https://crates.io/crates/ashpd) ![CI](https://github.com/bilelmoussaoui/ashpd/workflows/CI/badge.svg)

ASHPD, acronym of Aperture Science Handheld Portal Device is a Rust & [zbus](https://gitlab.freedesktop.org/dbus/zbus) wrapper of
the XDG portals DBus interfaces. The library aims to provide an easy way to
interact with the various portals defined per the [specifications](https://flatpak.github.io/xdg-desktop-portal/index.html).
It provides an alternative to the C library [https://github.com/flatpak/libportal](https://github.com/flatpak/libportal)

## Examples

Ask the compositor to pick a color

```rust,no_run
use ashpd::desktop::screenshot::ScreenshotProxy;
use ashpd::WindowIdentifier;

async fn run() -> ashpd::Result<()> {
    let connection = zbus::Connection::session().await?;
    let proxy = ScreenshotProxy::new(&connection).await?;

    let color = proxy.pick_color(&WindowIdentifier::default()).await?;
    println!("({}, {}, {})", color.red(), color.green(), color.blue());
    Ok(())
}
```

Start a PipeWire stream from the user's camera

```rust,no_run
use ashpd::desktop::camera::CameraProxy;

pub async fn run() -> ashpd::Result<()> {
    let connection = zbus::Connection::session().await?;
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
| log | Record various debug information using the `tracing` library |
| feature_gtk3 | Implement `From<Color>` for [`gdk3::RGBA`](https://gtk-rs.org/gtk3-rs/stable/latest/docs/gdk/struct.RGBA.html) |
|  | Provides `WindowIdentifier::from_window` that takes a [`IsA<gdk3::Window>`](https://gtk-rs.org/gtk3-rs/stable/latest/docs/gdk/struct.Window.html) |
| feature_gtk4 | Implement `From<Color>` for [`gdk4::RGBA`](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gdk4/struct.RGBA.html) |
|  | Provides `WindowIdentifier::from_native` that takes a [`IsA<gtk4::Native>`](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Native.html) |
| feature_pipewire | Provides `ashpd::desktop::camera::pipewire_node_id` that helps you retrieve the PipeWire Node ID to use with the file descriptor returned by the camera portal |
| raw_handle | Provides `WindowIdentifier::from_raw_handle` and `WindowIdentifier::as_raw_handle` for [raw-window-handle](https://lib.rs/crates/raw-window-handle) crate |


## Demo

The library comes with a [demo](./ashpd-demo) built using the [GTK 4 Rust bindings](https://gtk-rs.org/gtk4-rs) and previews most of the portals. It is meant as a test case for the portals (from a distributor perspective) and as a way for the developers to see which portals exists and how to integrate them into their application using ASHPD.

<a href="https://flathub.org/apps/details/com.belmoussaoui.ashpd.demo">
<img src="https://flathub.org/assets/badges/flathub-badge-i-en.png" width="190px" />
</a>

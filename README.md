# ASHPD

[![docs](https://docs.rs/ashpd/badge.svg)](https://docs.rs/ashpd/) [![crates.io](https://img.shields.io/crates/v/ashpd)](https://crates.io/crates/ashpd) ![CI](https://github.com/bilelmoussaoui/ashpd/workflows/CI/badge.svg)

ASHPD, acronym of Aperture Science Handheld Portal Device is a Rust & [zbus](https://gitlab.freedesktop.org/dbus/zbus) wrapper of
the XDG portals DBus interfaces. The library aims to provide an easy way to
interact with the various portals defined per the [specifications](https://flatpak.github.io/xdg-desktop-portal/index.html).
It provides an alternative to the C library [https://github.com/flatpak/libportal](https://github.com/flatpak/libportal)

## Examples

Ask the compositor to pick a color

```rust,no_run
use ashpd::desktop::Color;

async fn run() -> ashpd::Result<()> {
    let color = Color::pick().send().await?.response()?;
    println!("({}, {}, {})", color.red(), color.green(), color.blue());
    Ok(())
}
```

Start a PipeWire stream from the user's camera

```rust,no_run
use ashpd::desktop::camera::Camera;

pub async fn run() -> ashpd::Result<()> {
    let camera = Camera::new().await?;
    if camera.is_present().await? {
        camera.request_access().await?;
        let remote_fd = camera.open_pipe_wire_remote().await?;
        // pass the remote fd to GStreamer for example
    }
    Ok(())
}
```

## Optional features

| Feature | Description | Default |
| ---     | ----------- | ------- |
| tracing | Record various debug information using the `tracing` library | No |
| tokio | Enable tokio runtime on zbus dependency | No |
| async-std | Enable the use of the async-std runtime | Yes |
| gtk4 | Implement `From<Color>` for [`gdk4::RGBA`](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gdk4/struct.RGBA.html) Provides `WindowIdentifier::from_native` that takes a [`IsA<gtk4::Native>`](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Native.html) | No |
| gtk4_wayland |Provides `WindowIdentifier::from_native` that takes a [`IsA<gtk4::Native>`](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Native.html) with Wayland backend support only | No |
| gtk4_x11 |Provides `WindowIdentifier::from_native` that takes a [`IsA<gtk4::Native>`](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Native.html) with X11 backend support only | No |
| pipewire | Provides `ashpd::desktop::camera::pipewire_streams` that helps you retrieve the various camera streams associated with the retrieved file descriptor| No |
| raw_handle | Provides `WindowIdentifier::from_raw_handle` and `WindowIdentifier::as_raw_handle` for [raw-window-handle](https://lib.rs/crates/raw-window-handle) crate | No |
| wayland | Provides `WindowIdentifier::from_wayland` for [wayland-client](https://lib.rs/crates/wayland-client) crate | No |

## Demo

The library comes with a [demo](./ashpd-demo) built using the [GTK 4 Rust bindings](https://gtk-rs.org/gtk4-rs) and previews most of the portals. It is meant as a test case for the portals (from a distributor perspective) and as a way for the developers to see which portals exists and how to integrate them into their application using ASHPD.

<a href="https://flathub.org/apps/details/com.belmoussaoui.ashpd.demo">
<img src="https://flathub.org/assets/badges/flathub-badge-i-en.png" width="190px" />
</a>

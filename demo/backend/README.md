# Portal backend demo

ASHPD enables [writing portal backends](https://flatpak.github.io/xdg-desktop-portal/docs/writing-a-new-backend.html) that implement a subset of [backend interfaces](https://flatpak.github.io/xdg-desktop-portal/docs/impl-dbus-interfaces.html).

This demo provides a backend that registers the `org.freedesktop.impl.portal.desktop.ashpd-backend-demo` name and implements a handful of interfaces for demo purposes.

For testing purposes, the `$XDG_DESKTOP_PORTAL_DIR` environment variable can be set to instruct the frontend where to look for the portal and configuration files without installing them:

```shell
XDG_DESKTOP_PORTAL_DIR=$PWD/backend-demo/data /usr/libexec/xdg-desktop-portal -v -r
```

Note that this will temporarily override your system's defaults, so other existing backends (GNOME, KDE, …) won't be used as fallbacks, and as a consequence interfaces not implemented by this demo backend won't be available for applications to use.

If you intend to write an actual portal backend, you will need to ensure that its `.portal` file is properly installed under `{DATADIR}/xdg-desktop-portal/portals/`, and that your desktop environment's configuration file references it for the interfaces it implements.

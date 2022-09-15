use std::collections::HashMap;

use ashpd::{backend::settings::SettingsImpl, desktop::settings::Namespace};
use async_trait::async_trait;
use gtk::gio::prelude::*;
use zbus::zvariant::OwnedValue;

#[derive(Default)]
pub struct Settings;

const COLOR_SCHEME_SCHEMA: &str = "org.gnome.desktop.interface";

#[async_trait]
impl SettingsImpl for Settings {
    async fn read_all(&self, namespaces: Vec<String>) -> HashMap<String, Namespace> {
        // From ashpd: pub type Namespace = HashMap<String, OwnedValue>;
        log::debug!("IN ReadAll({namespaces:?})");
        let mut response = HashMap::<String, Namespace>::new();

        for namespace in &namespaces {
            if namespace_matches(namespace, "org.freedesktop.appearance") {
                let color_scheme = gsetting(COLOR_SCHEME_SCHEMA, "color-scheme");
                let color_scheme_namespace: Namespace =
                    Namespace::from([("color-scheme".to_string(), color_scheme)]);

                response.insert(
                    "org.freedesktop.appearance".to_string(),
                    color_scheme_namespace,
                );
            }

            if namespace_matches(namespace, "org.gnome.desktop.interface") {
                let color_scheme = gsetting(COLOR_SCHEME_SCHEMA, "color-scheme");
                let color_scheme_namespace: Namespace =
                    Namespace::from([("color-scheme".to_string(), color_scheme)]);

                response.insert(
                    "org.gnome.desktop.interface".to_string(),
                    color_scheme_namespace,
                );
            }
        }

        log::debug!("OUT ReadAll({response:?})");

        response
    }

    async fn read(&self, namespace: &str, key: &str) -> OwnedValue {
        log::debug!("IN Read({namespace}, {key})");
        let value = if namespace == "org.freedesktop.appearance" && key == "color-scheme" {
            gsetting(COLOR_SCHEME_SCHEMA, "color-scheme")
        } else {
            gsetting(namespace, key)
        };
        log::debug!("OUT Read({value:?})");
        value
    }
}

fn namespace_matches(namespace: &str, query: &str) -> bool {
    // TODO
    query.contains(namespace.trim_end_matches('*'))
}

fn gsetting(schema: &str, key: &str) -> OwnedValue {
    let settings = gtk::gio::Settings::new(schema);

    if key == "high-contrast" {
        settings.boolean(key).into()
    } else {
        (settings.enum_(key) as u32).into()
    }
}

// A tipical response from ReadAll(['org.gnome.*]) reads:
//
// {
//   'org.gnome.desktop.privacy': {'disable-microphone': <false>,
// 'disable-camera': <false>, 'old-files-age': <uint32 30>,
// 'remember-recent-files': <true>, 'disable-sound-output': <false>,
// 'send-software-usage-stats': <false>, 'report-technical-problems': <true>,
// 'remove-old-trash-files': <false>, 'remove-old-temp-files': <false>,
// 'privacy-screen': <false>, 'usb-protection': <true>, 'usb-protection-level':
// <'lockscreen'>, 'remember-app-usage': <true>, 'show-full-name-in-top-bar':
// <true>, 'hide-identity': <false>, 'recent-files-max-age': <-1>},
//   'org.gnome.desktop.a11y.interface': {'high-contrast': <false>},
//   'org.gnome.settings-daemon.plugins.xsettings': {'overrides': <@a{sv} {}>,
// 'disabled-gtk-modules': <@as []>, 'enabled-gtk-modules': <@as []>},
//   'org.gnome.desktop.interface': {'toolkit-accessibility': <false>,
// 'gtk-color-palette': <'black:white:gray50:red:purple:blue:light
// blue:green:yellow:orange:lavender:brown:goldenrod4:dodger blue:pink:light
// green:gray10:gray30:gray75:gray90'>, 'can-change-accels': <false>,
// 'color-scheme': <'prefer-dark'>, 'cursor-blink': <true>,
// 'clock-show-weekday': <false>, 'icon-theme': <'Adwaita'>,
// 'gtk-im-preedit-style': <'callback'>, 'scaling-factor': <uint32 0>,
// 'menus-have-tearoff': <false>, 'cursor-size': <24>, 'gtk-color-scheme': <''>,
// 'gtk-im-module': <''>, 'gtk-timeout-initial': <200>, 'gtk-theme':
// <'Adwaita-dark'>, 'clock-show-seconds': <false>, 'locate-pointer': <false>,
// 'clock-show-date': <true>, 'cursor-blink-time': <1200>, 'toolbar-icons-size':
// <'large'>, 'font-antialiasing': <'grayscale'>, 'gtk-timeout-repeat': <20>,
// 'toolbar-style': <'both-horiz'>, 'monospace-font-name': <'Source Code Pro
// 10'>, 'enable-hot-corners': <true>, 'overlay-scrolling': <true>,
// 'cursor-blink-timeout': <10>, 'gtk-key-theme': <'Default'>,
// 'toolbar-detachable': <false>, 'cursor-theme': <'Adwaita'>,
// 'avatar-directories': <@as []>, 'gtk-im-status-style': <'callback'>,
// 'menubar-detachable': <false>, 'text-scaling-factor': <1.0>,
// 'show-battery-percentage': <false>, 'clock-format': <'24h'>, 'menubar-accel':
// <'F10'>, 'font-rgba-order': <'rgb'>, 'font-hinting': <'slight'>,
// 'document-font-name': <'Cantarell 11'>, 'gtk-enable-primary-paste': <true>,
// 'enable-animations': <true>, 'font-name': <'Cantarell 11'>},   'org.gnome.
// desktop.sound': {'theme-name': <'__custom'>, 'event-sounds': <true>,
// 'input-feedback-sounds': <false>, 'allow-volume-above-100-percent': <true>},
//   'org.gnome.desktop.a11y': {'always-show-universal-access-status': <false>,
// 'always-show-text-caret': <false>},   'org.gnome.desktop.input-sources':
// {'mru-sources': <@a(ss) []>, 'show-all-sources': <false>, 'current': <uint32
// 0>, 'xkb-options': <@as []>, 'sources': <[('xkb', 'us+altgr-intl')]>,
// 'per-window': <false>},   'org.gnome.desktop.wm.preferences': {'theme':
// <'Adwaita'>, 'focus-new-windows': <'smart'>, 'num-workspaces': <4>,
// 'raise-on-click': <true>, 'disable-workarounds': <false>,
// 'titlebar-uses-system-font': <true>, 'titlebar-font': <'Cantarell Bold 11'>,
// 'resize-with-right-button': <false>, 'action-right-click-titlebar': <'menu'>,
// 'action-middle-click-titlebar': <'none'>, 'mouse-button-modifier':
// <'<Super>'>, 'auto-raise': <false>, 'workspace-names': <@as []>,
// 'action-double-click-titlebar': <'toggle-maximize'>, 'visual-bell-type':
// <'fullscreen-flash'>, 'focus-mode': <'click'>, 'button-layout':
// <'appmenu:close'>, 'auto-raise-delay': <500>, 'audible-bell': <true>,
// 'visual-bell': <false>},   'org.gnome.fontconfig': {'serial': <0>},
//   'org.gnome.desktop.privacy': {'disable-microphone': <false>,
// 'disable-camera': <false>, 'old-files-age': <uint32 30>,
// 'remember-recent-files': <true>, 'disable-sound-output': <false>,
// 'send-software-usage-stats': <false>, 'report-technical-problems': <true>,
// 'remove-old-trash-files': <false>, 'remove-old-temp-files': <false>,
// 'privacy-screen': <false>, 'usb-protection': <true>, 'usb-protection-level':
// <'lockscreen'>, 'remember-app-usage': <true>, 'show-full-name-in-top-bar':
// <true>, 'hide-identity': <false>, 'recent-files-max-age': <-1>},
//   'org.gnome.desktop.a11y.interface': {'high-contrast': <false>},
//   'org.gnome.settings-daemon.plugins.xsettings': {'overrides': <@a{sv} {}>,
// 'disabled-gtk-modules': <@as []>, 'enabled-gtk-modules': <@as []>},
//   'org.gnome.desktop.interface': {'toolkit-accessibility': <false>,
// 'gtk-color-palette': <'black:white:gray50:red:purple:blue:light
// blue:green:yellow:orange:lavender:brown:goldenrod4:dodger blue:pink:light
// green:gray10:gray30:gray75:gray90'>, 'can-change-accels': <false>,
// 'color-scheme': <'prefer-dark'>, 'cursor-blink': <true>,
// 'clock-show-weekday': <false>, 'icon-theme': <'Adwaita'>,
// 'gtk-im-preedit-style': <'callback'>, 'scaling-factor': <uint32 0>,
// 'menus-have-tearoff': <false>, 'cursor-size': <24>, 'gtk-color-scheme': <''>,
// 'gtk-im-module': <''>, 'gtk-timeout-initial': <200>, 'gtk-theme':
// <'Adwaita-dark'>, 'clock-show-seconds': <false>, 'locate-pointer': <false>,
// 'clock-show-date': <true>, 'cursor-blink-time': <1200>, 'toolbar-icons-size':
// <'large'>, 'font-antialiasing': <'grayscale'>, 'gtk-timeout-repeat': <20>,
// 'toolbar-style': <'both-horiz'>, 'monospace-font-name': <'Source Code Pro
// 10'>, 'enable-hot-corners': <true>, 'overlay-scrolling': <true>,
// 'cursor-blink-timeout': <10>, 'gtk-key-theme': <'Default'>,
// 'toolbar-detachable': <false>, 'cursor-theme': <'Adwaita'>,
// 'avatar-directories': <@as []>, 'gtk-im-status-style': <'callback'>,
// 'menubar-detachable': <false>, 'text-scaling-factor': <1.0>,
// 'show-battery-percentage': <false>, 'clock-format': <'24h'>, 'menubar-accel':
// <'F10'>, 'font-rgba-order': <'rgb'>, 'font-hinting': <'slight'>,
// 'document-font-name': <'Cantarell 11'>, 'gtk-enable-primary-paste': <true>,
// 'enable-animations': <true>, 'font-name': <'Cantarell 11'>},   'org.gnome.
// desktop.sound': {'theme-name': <'__custom'>, 'event-sounds': <true>,
// 'input-feedback-sounds': <false>, 'allow-volume-above-100-percent': <true>},
//   'org.gnome.desktop.a11y': {'always-show-universal-access-status': <false>,
// 'always-show-text-caret': <false>},   'org.gnome.desktop.input-sources':
// {'mru-sources': <@a(ss) []>, 'show-all-sources': <false>, 'current': <uint32
// 0>, 'xkb-options': <@as []>, 'sources': <[('xkb', 'us+altgr-intl')]>,
// 'per-window': <false>},   'org.gnome.desktop.wm.preferences': {'theme':
// <'Adwaita'>, 'focus-new-windows': <'smart'>, 'num-workspaces': <4>,
// 'raise-on-click': <true>, 'disable-workarounds': <false>,
// 'titlebar-uses-system-font': <true>, 'titlebar-font': <'Cantarell Bold 11'>,
// 'resize-with-right-button': <false>, 'action-right-click-titlebar': <'menu'>,
// 'action-middle-click-titlebar': <'none'>, 'mouse-button-modifier':
// <'<Super>'>, 'auto-raise': <false>, 'workspace-names': <@as []>,
// 'action-double-click-titlebar': <'toggle-maximize'>, 'visual-bell-type':
// <'fullscreen-flash'>, 'focus-mode': <'click'>, 'button-layout':
// <'appmenu:close'>, 'auto-raise-delay': <500>, 'audible-bell': <true>,
// 'visual-bell': <false>},   'org.gnome.fontconfig': {'serial': <0>},
// }

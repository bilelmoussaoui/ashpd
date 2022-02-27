//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::remote_desktop::{DeviceType, KeyState, RemoteDesktopProxy};
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = RemoteDesktopProxy::new(&connection).await?;
//!
//!     let session = proxy.create_session().await?;
//!
//!     proxy.select_devices(&session, DeviceType::Keyboard | DeviceType::Pointer).await?;
//!
//!     let (devices, _) = proxy.start(&session, &WindowIdentifier::default()).await?;
//!     println!("{:#?}", devices);
//!
//!     // 13 for Enter key code
//!     proxy.notify_keyboard_keycode(&session, 13, KeyState::Pressed).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! You can also use the Remote Desktop portal with the ScreenCast one. In order to do so,
//! you need to call [`ScreenCastProxy::select_sources()`](crate::desktop::screencast::ScreenCastProxy::select_sources)
//! on the session created with [`RemoteDesktopProxy::create_session()`](crate::desktop::remote_desktop::RemoteDesktopProxy::create_session)
//!
//! ```rust,no_run
//! use ashpd::desktop::remote_desktop::{DeviceType, KeyState, RemoteDesktopProxy};
//! use ashpd::desktop::screencast::{CursorMode, PersistMode, ScreenCastProxy, SourceType};
//! use ashpd::WindowIdentifier;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = RemoteDesktopProxy::new(&connection).await?;
//!     let screencast = ScreenCastProxy::new(&connection).await?;
//!     let identifier = WindowIdentifier::default();
//!
//!     let session = proxy.create_session().await?;
//!
//!     proxy.select_devices(&session, DeviceType::Keyboard | DeviceType::Pointer).await?;
//!     screencast
//!         .select_sources(
//!             &session,
//!             CursorMode::Metadata.into(),
//!             SourceType::Monitor | SourceType::Window,
//!             true,
//!             None,
//!             PersistMode::DoNot,
//!         )
//!         .await?;
//!
//!     let (devices, streams) = proxy.start(&session, &identifier).await?;
//!     println!("{:#?}", devices);
//!     println!("{:#?}", streams);
//!
//!     // 13 for Enter key code
//!     proxy.notify_keyboard_keycode(&session, 13, KeyState::Pressed).await?;
//!
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;

use enumflags2::{bitflags, BitFlags};
use futures::TryFutureExt;
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::zvariant::{DeserializeDict, SerializeDict, Type, Value};

use super::{screencast::Stream, HandleToken, SessionProxy, DESTINATION, PATH};

use crate::{
    helpers::{call_basic_response_method, call_method, call_request_method},
    Error, WindowIdentifier,
};

#[derive(Serialize_repr, Deserialize_repr, Copy, Clone, PartialEq, Debug, Type)]
#[doc(alias = "XdpKeyState")]
/// The keyboard key state.
#[repr(u32)]
pub enum KeyState {
    #[doc(alias = "XDP_KEY_PRESSED")]
    /// The key is pressed.
    Pressed = 0,
    #[doc(alias = "XDP_KEY_RELEASED")]
    /// The key is released..
    Released = 1,
}

#[bitflags]
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Copy, Type)]
#[repr(u32)]
#[doc(alias = "XdpDeviceType")]
/// A bit flag for the available devices.
pub enum DeviceType {
    #[doc(alias = "XDP_DEVICE_KEYBOARD")]
    /// A keyboard.
    Keyboard,
    #[doc(alias = "XDP_DEVICE_POINTER")]
    /// A mouse pointer.
    Pointer,
    #[doc(alias = "XDP_DEVICE_TOUCHSCREEN")]
    /// A touchscreen
    Touchscreen,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[doc(alias = "XdpDiscreteAxis")]
#[repr(u32)]
/// The available axis.
pub enum Axis {
    #[doc(alias = "XDP_AXIS_VERTICAL_SCROLL")]
    /// Vertical axis.
    Vertical = 0,
    #[doc(alias = "XDP_AXIS_HORIZONTAL_SCROLL")]
    /// Horizontal axis.
    Horizontal = 1,
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`RemoteDesktopProxy::create_session`] request.
#[zvariant(signature = "dict")]
struct CreateRemoteOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// A string that will be used as the last element of the session handle.
    session_handle_token: HandleToken,
}

#[derive(SerializeDict, DeserializeDict, Type, Debug)]
/// A response to a [`RemoteDesktopProxy::create_session`] request.
#[zvariant(signature = "dict")]
struct CreateSession {
    // TODO: investigate why this doesn't return an ObjectPath
    // replace with an ObjectPath once https://github.com/flatpak/xdg-desktop-portal/pull/609's merged
    /// A string that will be used as the last element of the session handle.
    session_handle: String,
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`RemoteDesktopProxy::select_devices`] request.
#[zvariant(signature = "dict")]
struct SelectDevicesOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// The device types to request remote controlling of. Default is all.
    types: Option<BitFlags<DeviceType>>,
}

impl SelectDevicesOptions {
    /// Sets the device types to request remote controlling of.
    pub fn types(mut self, types: BitFlags<DeviceType>) -> Self {
        self.types = Some(types);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// Specified options for a [`RemoteDesktopProxy::start`] request.
#[zvariant(signature = "dict")]
struct StartRemoteOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
}

#[derive(SerializeDict, DeserializeDict, Type, Debug, Default)]
/// A response to a [`RemoteDesktopProxy::select_devices`] request.
#[zvariant(signature = "dict")]
struct SelectedDevices {
    /// The selected devices.
    devices: BitFlags<DeviceType>,
    /// The selected streams if a ScreenCast portal is used on the same session
    streams: Option<Vec<Stream>>,
}

/// The interface lets sandboxed applications create remote desktop sessions.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.RemoteDesktop`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.RemoteDesktop).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.RemoteDesktop")]
pub struct RemoteDesktopProxy<'a>(zbus::Proxy<'a>);

impl<'a> RemoteDesktopProxy<'a> {
    /// Create a new instance of [`RemoteDesktopProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<RemoteDesktopProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.RemoteDesktop")?
            .path(PATH)?
            .destination(DESTINATION)?
            .build()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::Proxy<'_> {
        &self.0
    }

    /// Create a remote desktop session.
    /// A remote desktop session is used to allow remote controlling a desktop
    /// session. It can also be used together with a screen cast session.
    ///
    /// # Specifications
    ///
    /// See also [`CreateSession`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-RemoteDesktop.CreateSession).
    #[doc(alias = "CreateSession")]
    #[doc(alias = "xdp_portal_create_remote_desktop_session")]
    pub async fn create_session(&self) -> Result<SessionProxy<'a>, Error> {
        let options = CreateRemoteOptions::default();
        let (session, proxy) = futures::try_join!(
            call_request_method::<CreateSession, _>(
                self.inner(),
                &options.handle_token,
                "CreateSession",
                &options
            )
            .into_future(),
            SessionProxy::from_unique_name(
                self.inner().connection(),
                &options.session_handle_token
            )
            .into_future()
        )?;
        assert_eq!(proxy.inner().path().as_str(), &session.session_handle);
        Ok(proxy)
    }

    /// Select input devices to remote control.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`RemoteDesktopProxy::create_session`].
    /// * `types` - The device types to request remote controlling of.
    ///
    /// # Specifications
    ///
    /// See also [`SelectDevices`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-RemoteDesktop.SelectDevices).
    #[doc(alias = "SelectDevices")]
    pub async fn select_devices(
        &self,
        session: &SessionProxy<'_>,
        types: BitFlags<DeviceType>,
    ) -> Result<(), Error> {
        let options = SelectDevicesOptions::default().types(types);
        call_basic_response_method(
            self.inner(),
            &options.handle_token,
            "SelectDevices",
            &(session, &options),
        )
        .await
    }

    ///  Start the remote desktop session.
    ///
    /// This will typically result in the portal presenting a dialog letting
    /// the user select what to share, including devices and optionally screen
    /// content if screen cast sources was selected.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`RemoteDesktopProxy::create_session`].
    /// * `identifier` - The application window identifier.
    ///
    /// # Specifications
    ///
    /// See also [`Start`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-RemoteDesktop.Start).
    #[doc(alias = "Start")]
    pub async fn start(
        &self,
        session: &SessionProxy<'_>,
        identifier: &WindowIdentifier,
    ) -> Result<(BitFlags<DeviceType>, Vec<Stream>), Error> {
        let options = StartRemoteOptions::default();
        let response: SelectedDevices = call_request_method(
            self.inner(),
            &options.handle_token,
            "Start",
            &(session, &identifier, &options),
        )
        .await?;
        Ok((response.devices, response.streams.unwrap_or_default()))
    }

    /// Notify keyboard code.
    ///
    /// **Note** only works if [`DeviceType::Keyboard`] access was provided
    /// after starting the session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`RemoteDesktopProxy::create_session`].
    /// * `keycode` - Keyboard code that was pressed or released.
    /// * `state` - The new state of the keyboard code.
    ///
    /// # Specifications
    ///
    /// See also [`NotifyKeyboardKeycode`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-RemoteDesktop.NotifyKeyboardKeycode).
    #[doc(alias = "NotifyKeyboardKeycode")]
    pub async fn notify_keyboard_keycode(
        &self,
        session: &SessionProxy<'_>,
        keycode: i32,
        state: KeyState,
    ) -> Result<(), Error> {
        // The `notify` methods don't take any options for now
        // see https://github.com/flatpak/xdg-desktop-portal/blob/master/src/remote-desktop.c#L723
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(
            self.inner(),
            "NotifyKeyboardKeycode",
            &(session, options, keycode, state),
        )
        .await
    }

    /// Notify keyboard symbol.
    ///
    /// **Note** only works if [`DeviceType::Keyboard`] access was provided
    /// after starting the session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`RemoteDesktopProxy::create_session`].
    /// * `keysym` - Keyboard symbol that was pressed or released.
    /// * `state` - The new state of the keyboard code.
    ///
    /// # Specifications
    ///
    /// See also [`NotifyKeyboardKeysym`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-RemoteDesktop.NotifyKeyboardKeysym).
    #[doc(alias = "NotifyKeyboardKeysym")]
    pub async fn notify_keyboard_keysym(
        &self,
        session: &SessionProxy<'_>,
        keysym: i32,
        state: KeyState,
    ) -> Result<(), Error> {
        // The `notify` methods don't take any options for now
        // see https://github.com/flatpak/xdg-desktop-portal/blob/master/src/remote-desktop.c#L723
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(
            self.inner(),
            "NotifyKeyboardKeysym",
            &(session, options, keysym, state),
        )
        .await
    }

    /// Notify about a new touch up event.
    ///
    /// **Note** only works if [`DeviceType::Touchscreen`] access was provided
    /// after starting the session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`RemoteDesktopProxy::create_session`].
    /// * `slot` - Touch slot where touch point appeared.
    ///
    /// # Specifications
    ///
    /// See also [`NotifyTouchUp`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-RemoteDesktop.NotifyTouchUp).
    #[doc(alias = "NotifyTouchUp")]
    pub async fn notify_touch_up(
        &self,
        session: &SessionProxy<'_>,
        slot: u32,
    ) -> Result<(), Error> {
        // The `notify` methods don't take any options for now
        // see https://github.com/flatpak/xdg-desktop-portal/blob/master/src/remote-desktop.c#L723
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(self.inner(), "NotifyTouchUp", &(session, options, slot)).await
    }

    /// Notify about a new touch down event.
    /// The (x, y) position represents the new touch point position in the
    /// streams logical coordinate space.
    ///
    /// **Note** only works if [`DeviceType::Touchscreen`] access was provided
    /// after starting the session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`RemoteDesktopProxy::create_session`].
    /// * `stream` - The PipeWire stream node the coordinate is relative to.
    /// * `slot` - Touch slot where touch point appeared.
    /// * `x` - Touch down x coordinate.
    /// * `y` - Touch down y coordinate.
    ///
    /// # Specifications
    ///
    /// See also [`NotifyTouchDown`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-RemoteDesktop.NotifyTouchDown).
    #[doc(alias = "NotifyTouchDown")]
    pub async fn notify_touch_down(
        &self,
        session: &SessionProxy<'_>,
        stream: u32,
        slot: u32,
        x: f64,
        y: f64,
    ) -> Result<(), Error> {
        // The `notify` methods don't take any options for now
        // see https://github.com/flatpak/xdg-desktop-portal/blob/master/src/remote-desktop.c#L723
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(
            self.inner(),
            "NotifyTouchDown",
            &(session, options, stream, slot, x, y),
        )
        .await
    }

    /// Notify about a new touch motion event.
    /// The (x, y) position represents where the touch point position in the
    /// streams logical coordinate space moved.
    ///
    /// **Note** only works if [`DeviceType::Touchscreen`] access was provided
    /// after starting the session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`RemoteDesktopProxy::create_session`].
    /// * `stream` - The PipeWire stream node the coordinate is relative to.
    /// * `slot` - Touch slot where touch point appeared.
    /// * `x` - Touch motion x coordinate.
    /// * `y` - Touch motion y coordinate.
    ///
    /// # Specifications
    ///
    /// See also [`NotifyTouchMotion`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-RemoteDesktop.NotifyTouchMotion).
    #[doc(alias = "NotifyTouchMotion")]
    pub async fn notify_touch_motion(
        &self,
        session: &SessionProxy<'_>,
        stream: u32,
        slot: u32,
        x: f64,
        y: f64,
    ) -> Result<(), Error> {
        // The `notify` methods don't take any options for now
        // see https://github.com/flatpak/xdg-desktop-portal/blob/master/src/remote-desktop.c#L723
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(
            self.inner(),
            "NotifyTouchMotion",
            &(session, options, stream, slot, x, y),
        )
        .await
    }

    /// Notify about a new absolute pointer motion event.
    /// The (x, y) position represents the new pointer position in the streams
    /// logical coordinate space.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`RemoteDesktopProxy::create_session`].
    /// * `stream` - The PipeWire stream node the coordinate is relative to.
    /// * `x` - Pointer motion x coordinate.
    /// * `y` - Pointer motion y coordinate.
    ///
    /// # Specifications
    ///
    /// See also [`NotifyPointerMotionAbsolute`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-RemoteDesktop.NotifyPointerMotionAbsolute).
    #[doc(alias = "NotifyPointerMotionAbsolute")]
    pub async fn notify_pointer_motion_absolute(
        &self,
        session: &SessionProxy<'_>,
        stream: u32,
        x: f64,
        y: f64,
    ) -> Result<(), Error> {
        // The `notify` methods don't take any options for now
        // see https://github.com/flatpak/xdg-desktop-portal/blob/master/src/remote-desktop.c#L723
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(
            self.inner(),
            "NotifyPointerMotionAbsolute",
            &(session, options, stream, x, y),
        )
        .await
    }

    /// Notify about a new relative pointer motion event.
    /// The (dx, dy) vector represents the new pointer position in the streams
    /// logical coordinate space.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`RemoteDesktopProxy::create_session`].
    /// * `dx` - Relative movement on the x axis.
    /// * `dy` - Relative movement on the y axis.
    ///
    /// # Specifications
    ///
    /// See also [`NotifyPointerMotion`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-RemoteDesktop.NotifyPointerMotion).
    #[doc(alias = "NotifyPointerMotion")]
    pub async fn notify_pointer_motion(
        &self,
        session: &SessionProxy<'_>,
        dx: f64,
        dy: f64,
    ) -> Result<(), Error> {
        // The `notify` methods don't take any options for now
        // see https://github.com/flatpak/xdg-desktop-portal/blob/master/src/remote-desktop.c#L723
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(
            self.inner(),
            "NotifyPointerMotion",
            &(session, options, dx, dy),
        )
        .await
    }

    /// Notify pointer button.
    /// The pointer button is encoded according to Linux Evdev button codes.
    ///
    ///
    /// **Note** only works if [`DeviceType::Pointer`] access was provided after
    /// starting the session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`RemoteDesktopProxy::create_session`].
    /// * `button` - The pointer button was pressed or released.
    /// * `state` - The new state of the keyboard code.
    ///
    /// # Specifications
    ///
    /// See also [`NotifyPointerButton`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-RemoteDesktop.NotifyPointerButton).
    #[doc(alias = "NotifyPointerButton")]
    pub async fn notify_pointer_button(
        &self,
        session: &SessionProxy<'_>,
        button: i32,
        state: KeyState,
    ) -> Result<(), Error> {
        // The `notify` methods don't take any options for now
        // see https://github.com/flatpak/xdg-desktop-portal/blob/master/src/remote-desktop.c#L723
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(
            self.inner(),
            "NotifyPointerButton",
            &(session, options, button, state),
        )
        .await
    }

    /// Notify pointer axis discrete.
    ///
    /// **Note** only works if [`DeviceType::Pointer`] access was provided after
    /// starting the session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`RemoteDesktopProxy::create_session`].
    /// * `axis` - The axis that was scrolled.
    ///
    /// # Specifications
    ///
    /// See also [`NotifyPointerAxisDiscrete`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-RemoteDesktop.NotifyPointerAxisDiscrete).
    #[doc(alias = "NotifyPointerAxisDiscrete")]
    pub async fn notify_pointer_axis_discrete(
        &self,
        session: &SessionProxy<'_>,
        axis: Axis,
        steps: i32,
    ) -> Result<(), Error> {
        // The `notify` methods don't take any options for now
        // see https://github.com/flatpak/xdg-desktop-portal/blob/master/src/remote-desktop.c#L723
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(
            self.inner(),
            "NotifyPointerAxisDiscrete",
            &(session, options, axis, steps),
        )
        .await
    }

    /// Notify pointer axis.
    /// The axis movement from a "smooth scroll" device, such as a touchpad.
    /// When applicable, the size of the motion delta should be equivalent to
    /// the motion vector of a pointer motion done using the same advice.
    ///
    ///
    /// **Note** only works if [`DeviceType::Pointer`] access was provided after
    /// starting the session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`], created with
    ///   [`create_session()`][`RemoteDesktopProxy::create_session`].
    /// * `dx` - Relative axis movement on the x axis.
    /// * `dy` - Relative axis movement on the y axis.
    /// * `finish` - Whether it is the last axis event.
    ///
    /// # Specifications
    ///
    /// See also [`NotifyPointerAxis`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-RemoteDesktop.NotifyPointerAxis).
    #[doc(alias = "NotifyPointerAxis")]
    pub async fn notify_pointer_axis(
        &self,
        session: &SessionProxy<'_>,
        dx: f64,
        dy: f64,
        finish: bool,
    ) -> Result<(), Error> {
        // see https://github.com/flatpak/xdg-desktop-portal/blob/master/src/remote-desktop.c#L911
        let mut options: HashMap<&str, Value<'_>> = HashMap::new();
        options.insert("finish", Value::Bool(finish));
        call_method(
            self.inner(),
            "NotifyPointerAxis",
            &(session, options, dx, dy),
        )
        .await
    }

    /// Available source types.
    ///
    /// # Specifications
    ///
    /// See also [`AvailableDeviceTypes`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-property-org-freedesktop-portal-RemoteDesktop.AvailableDeviceTypes).
    #[doc(alias = "AvailableDeviceTypes")]
    pub async fn available_device_types(&self) -> Result<BitFlags<DeviceType>, Error> {
        self.inner()
            .get_property::<BitFlags<DeviceType>>("AvailableDeviceTypes")
            .await
            .map_err(From::from)
    }
}

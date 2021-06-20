//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::remote_desktop::{DeviceType, KeyState, RemoteDesktopProxy};
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = RemoteDesktopProxy::new(&connection).await?;
//!
//!     let session = proxy.create_session().await?;
//!
//!     proxy.select_devices(&session, DeviceType::Keyboard | DeviceType::Pointer).await?;
//!
//!     let devices = proxy.start(&session, Default::default()).await?;
//!     println!("{:#?}", devices);
//!
//!     // 13 for Enter key code
//!     proxy.notify_keyboard_keycode(&session, 13, KeyState::Pressed).await?;
//!
//!     Ok(())
//! }
//! ```
use enumflags2::BitFlags;
use futures::TryFutureExt;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use zvariant::{OwnedObjectPath, Value};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

use crate::{
    helpers::{call_basic_response_method, call_method, call_request_method, property},
    Error, WindowIdentifier,
};

use super::{HandleToken, SessionProxy, DESTINATION, PATH};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
/// The keyboard key state.
pub enum KeyState {
    /// The key is pressed.
    Pressed = 0,
    /// The key is released..
    Released = 1,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, BitFlags, Clone, Copy, Type)]
#[repr(u32)]
/// A bit flag for the available devices.
pub enum DeviceType {
    /// A keyboard.
    Keyboard = 1,
    /// A mouse pointer.
    Pointer = 2,
    /// A touchscreen
    Touchscreen = 4,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
/// The available axis.
pub enum Axis {
    /// Vertical axis.
    Vertical = 0,
    /// Horizontal axis.
    Horizontal = 1,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a [`RemoteDesktopProxy::create_session`] request.
struct CreateRemoteOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// A string that will be used as the last element of the session handle.
    session_handle_token: HandleToken,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// A response to a [`RemoteDesktopProxy::create_session`] request.
struct CreateSession {
    /// A string that will be used as the last element of the session handle.
    session_handle: OwnedObjectPath,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a [`RemoteDesktopProxy::select_devices`] request.
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

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a [`RemoteDesktopProxy::start`] request.
struct StartRemoteOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// A response to a [`RemoteDesktopProxy::select_devices`] request.
struct SelectedDevices {
    /// The selected devices.
    pub devices: BitFlags<DeviceType>,
}

/// The interface lets sandboxed applications create remote desktop sessions.
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.RemoteDesktop")]
pub struct RemoteDesktopProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> RemoteDesktopProxy<'a> {
    /// Create a new instance of [`RemoteDesktopProxy`].
    pub async fn new(
        connection: &zbus::azync::Connection,
    ) -> Result<RemoteDesktopProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.RemoteDesktop")
            .path(PATH)?
            .destination(DESTINATION)
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::azync::Proxy<'_> {
        &self.0
    }

    /// Create a remote desktop session.
    /// A remote desktop session is used to allow remote controlling a desktop
    /// session. It can also be used together with a screen cast session.
    #[doc(alias = "CreateSession")]
    pub async fn create_session(&self) -> Result<SessionProxy<'a>, Error> {
        let options = CreateRemoteOptions::default();
        let (proxy, session) = futures::try_join!(
            SessionProxy::from_unique_name(self.0.connection(), &options.session_handle_token)
                .into_future(),
            call_request_method::<CreateSession, CreateRemoteOptions>(
                &self.0,
                &options.handle_token,
                "CreateSession",
                &(&options)
            )
            .into_future()
        )?;
        assert_eq!(
            proxy.inner().path().clone(),
            session.session_handle.into_inner()
        );
        Ok(proxy)
    }

    /// Select input devices to remote control.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`].
    /// * `types` - The device types to request remote controlling of.
    #[doc(alias = "SelectDevices")]
    pub async fn select_devices(
        &self,
        session: &SessionProxy<'_>,
        types: BitFlags<DeviceType>,
    ) -> Result<(), Error> {
        let options = SelectDevicesOptions::default().types(types);
        call_basic_response_method(
            &self.0,
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
    /// * `session` - A [`SessionProxy`].
    /// * `parent_window` - The application window identifier.
    #[doc(alias = "Start")]
    pub async fn start(
        &self,
        session: &SessionProxy<'_>,
        parent_window: WindowIdentifier,
    ) -> Result<BitFlags<DeviceType>, Error> {
        let options = StartRemoteOptions::default();
        let response: SelectedDevices = call_request_method(
            &self.0,
            &options.handle_token,
            "Start",
            &(session, parent_window, &options),
        )
        .await?;
        Ok(response.devices)
    }

    /// Notify keyboard code.
    /// May only be called if KEYBOARD access was provided after starting the
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`].
    /// * `keycode` - Keyboard code that was pressed or released.
    /// * `state` - The new state of the keyboard code.
    #[doc(alias = "NotifyKeyboardKeycode")]
    pub async fn notify_keyboard_keycode(
        &self,
        session: &SessionProxy<'_>,
        keycode: i32,
        state: KeyState,
    ) -> Result<(), Error> {
        // FIXME: figure out the options we can take here
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(
            &self.0,
            "NotifyKeyboardKeycode",
            &(session, options, keycode, state),
        )
        .await
    }

    /// Notify keyboard symbol.
    /// May only be called if KEYBOARD access was provided after starting the
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`].
    /// * `keysym` - Keyboard symbol that was pressed or released.
    /// * `state` - The new state of the keyboard code.
    #[doc(alias = "NotifyKeyboardKeysym")]
    pub async fn notify_keyboard_keysym(
        &self,
        session: &SessionProxy<'_>,
        keysym: i32,
        state: KeyState,
    ) -> Result<(), Error> {
        // FIXME: figure out the options we can take here
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(
            &self.0,
            "NotifyKeyboardKeysym",
            &(session, options, keysym, state),
        )
        .await
    }

    /// Notify about a new touch up event.
    ///
    /// May only be called if TOUCHSCREEN access was provided after starting the
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`].
    /// * `slot` - Touch slot where touch point appeared.
    #[doc(alias = "NotifyTouchUp")]
    pub async fn notify_touch_up(
        &self,
        session: &SessionProxy<'_>,
        slot: u32,
    ) -> Result<(), Error> {
        // FIXME: figure out the options we can take here
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(&self.0, "NotifyTouchUp", &(session, options, slot)).await
    }

    /// Notify about a new touch down event.
    /// The (x, y) position represents the new touch point position in the
    /// streams logical coordinate space.
    ///
    /// May only be called if TOUCHSCREEN access was provided after starting the
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`].
    /// * `stream` - The PipeWire stream node the coordinate is relative to.
    /// * `slot` - Touch slot where touch point appeared.
    /// * `x` - Touch down x coordinate.
    /// * `y` - Touch down y coordinate.
    #[doc(alias = "NotifyTouchDown")]
    pub async fn notify_touch_down(
        &self,
        session: &SessionProxy<'_>,
        stream: u32,
        slot: u32,
        x: f64,
        y: f64,
    ) -> Result<(), Error> {
        // FIXME: figure out the options we can take here
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(
            &self.0,
            "NotifyTouchDown",
            &(session, options, stream, slot, x, y),
        )
        .await
    }

    /// Notify about a new touch motion event.
    /// The (x, y) position represents where the touch point position in the
    /// streams logical coordinate space moved.
    ///
    /// May only be called if TOUCHSCREEN access was provided after starting the
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`].
    /// * `stream` - The PipeWire stream node the coordinate is relative to.
    /// * `slot` - Touch slot where touch point appeared.
    /// * `x` - Touch motion x coordinate.
    /// * `y` - Touch motion y coordinate.
    #[doc(alias = "NotifyTouchMotion")]
    pub async fn notify_touch_motion(
        &self,
        session: &SessionProxy<'_>,
        stream: u32,
        slot: u32,
        x: f64,
        y: f64,
    ) -> Result<(), Error> {
        // FIXME: figure out the options we can take here
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(
            &self.0,
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
    /// * `session` - A [`SessionProxy`].
    /// * `stream` - The PipeWire stream node the coordinate is relative to.
    /// * `x` - Pointer motion x coordinate.
    /// * `y` - Pointer motion y coordinate.
    #[doc(alias = "NotifyPointerMotionAbsolute")]
    pub async fn notify_pointer_motion_absolute(
        &self,
        session: &SessionProxy<'_>,
        stream: u32,
        x: f64,
        y: f64,
    ) -> Result<(), Error> {
        // FIXME: figure out the options we can take here
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(
            &self.0,
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
    /// * `session` - A [`SessionProxy`].
    /// * `dx` - Relative movement on the x axis.
    /// * `dy` - Relative movement on the y axis.
    #[doc(alias = "NotifyPointerMotion")]
    pub async fn notify_pointer_motion(
        &self,
        session: &SessionProxy<'_>,
        dx: f64,
        dy: f64,
    ) -> Result<(), Error> {
        // FIXME: figure out the options we can take here
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(&self.0, "NotifyPointerMotion", &(session, options, dx, dy)).await
    }

    /// Notify pointer button.
    /// The pointer button is encoded according to Linux Evdev button codes.
    ///
    ///  May only be called if POINTER access was provided after starting the
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`].
    /// * `button` - The pointer button was pressed or released.
    /// * `state` - The new state of the keyboard code.
    #[doc(alias = "NotifyPointerButton")]
    pub async fn notify_pointer_button(
        &self,
        session: &SessionProxy<'_>,
        button: i32,
        state: KeyState,
    ) -> Result<(), Error> {
        // FIXME: figure out the options we can take here
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(
            &self.0,
            "NotifyPointerButton",
            &(session, options, button, state),
        )
        .await
    }

    /// Notify pointer axis discrete.
    /// May only be called if POINTER access was provided after starting the
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`].
    /// * `axis` - The axis that was scrolled.
    #[doc(alias = "NotifyPointerAxisDiscrete")]
    pub async fn notify_pointer_axis_discrete(
        &self,
        session: &SessionProxy<'_>,
        axis: Axis,
        steps: i32,
    ) -> Result<(), Error> {
        // FIXME: figure out the options we can take here
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(
            &self.0,
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
    /// May only be called if POINTER access was provided after starting the
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session` - A [`SessionProxy`].
    /// * `dx` - Relative axis movement on the x axis.
    /// * `dy` - Relative axis movement on the y axis.
    #[doc(alias = "NotifyPointerAxis")]
    pub async fn notify_pointer_axis(
        &self,
        session: &SessionProxy<'_>,
        dx: f64,
        dy: f64,
    ) -> Result<(), Error> {
        // FIXME: figure out the options we can take here
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        call_method(&self.0, "NotifyPointerAxis", &(session, options, dx, dy)).await
    }

    /// Available source types.
    #[doc(alias = "AvailableDeviceTypes")]
    pub async fn available_device_types(&self) -> Result<BitFlags<DeviceType>, Error> {
        property(&self.0, "AvailableDeviceTypes").await
    }

    /// The version of this DBus interface.
    pub async fn version(&self) -> Result<u32, Error> {
        property(&self.0, "version").await
    }
}

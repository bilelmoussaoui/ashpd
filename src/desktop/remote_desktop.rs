use crate::WindowIdentifier;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{ObjectPath, OwnedObjectPath, Value};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
pub enum KeyState {
    Pressed = 0,
    Released = 1,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
pub enum DeviceType {
    Keyboard = 1,
    Pointeur = 2,
    Touchscreen = 4,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Type)]
#[repr(u32)]
pub enum Axis {
    Vertical = 0,
    Horizontal = 1,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a create a remote session request.
pub struct CreateRemoteOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
    /// A string that will be used as the last element of the session handle.
    pub session_handle_token: Option<String>,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options on a select devices request.
pub struct SelectDevicesOptions {
    /// A string that will be used as the last element of the handle.
    pub handle_token: Option<String>,
    /// The device types to request remote controlling of. Default is all.
    pub types: Option<DeviceType>,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a start remote desktop request.
pub struct StartRemoteOptions {
    /// A string that will be used as the last element of the handle. Must be a valid object path element.
    pub handle_token: Option<String>,
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.RemoteDesktop",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications create remote desktop sessions.
trait RemoteDesktop {
    /// Create a remote desktop session.
    /// A remote desktop session is used to allow remote controlling a desktop session.
    /// It can also be used together with a screen cast session
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `options` - A [`CreateRemoteOptions`]
    ///
    /// [`CreateRemoteOptions`]: ./struct.CreateRemoteOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn create_session(&self, options: CreateRemoteOptions) -> Result<OwnedObjectPath>;

    /// Notify keyboard code
    /// May only be called if KEYBOARD access was provided after starting the session.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - The [`Session`] object handle
    /// * `options` - ?
    /// * `keycode` - Keyboard code that was pressed or released
    /// * `state` - The new state of the keyboard code
    ///
    /// [`Session`]: ../session/struct.SessionProxy.html
    fn notify_keyboard_keycode(
        &self,
        session_handle: ObjectPath,
        options: HashMap<&str, Value>,
        keycode: i32,
        state: KeyState,
    ) -> Result<()>;

    /// Notify keyboard symbol
    /// May only be called if KEYBOARD access was provided after starting the session.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - The [`Session`] object handle
    /// * `options` - ?
    /// * `keysym` - Keyboard symbol that was pressed or released
    /// * `state` - The new state of the keyboard code
    ///
    /// [`Session`]: ../session/struct.SessionProxy.html
    fn notify_keyboard_keysym(
        &self,
        session_handle: ObjectPath,
        options: HashMap<&str, Value>,
        keysym: i32,
        state: KeyState,
    ) -> Result<()>;

    /// Notify pointer axis
    /// The axis movement from a 'smooth scroll' device, such as a touchpad.
    /// When applicable, the size of the motion delta should be equivalent to
    /// the motion vector of a pointer motion done using the same advice.
    ///
    /// May only be called if POINTER access was provided after starting the session.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - The [`Session`] object handle
    /// * `options` - ?
    /// * `dx` - Relative axis movement on the x axis
    /// * `dy` - Relative axis movement on the y axis
    ///
    /// [`Session`]: ../session/struct.SessionProxy.html
    fn notify_pointer_axis(
        &self,
        session_handle: ObjectPath,
        options: HashMap<&str, Value>,
        dx: f64,
        dy: f64,
    ) -> Result<()>;

    /// Notify pointer axis discrete
    /// May only be called if POINTER access was provided after starting the session.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - The [`Session`] object handle
    /// * `options` - ?
    /// * `axis` - The axis that was scrolled
    ///
    /// [`Session`]: ../session/struct.SessionProxy.html
    fn notify_pointer_axis_discrete(
        &self,
        session_handle: ObjectPath,
        options: HashMap<&str, Value>,
        axis: Axis,
        steps: i32,
    ) -> Result<()>;

    /// Notify pointer button
    /// The pointer button is encoded according to Linux Evdev button codes.
    ///
    ///  May only be called if POINTER access was provided after starting the session.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - The [`Session`] object handle
    /// * `options` - ?
    /// * `button` - The pointer button was pressed or released
    /// * `state` - The new state of the keyboard code
    ///
    /// [`Session`]: ../session/struct.SessionProxy.html
    fn notify_pointer_button(
        &self,
        session_handle: ObjectPath,
        options: HashMap<&str, Value>,
        button: i32,
        state: KeyState,
    ) -> Result<()>;

    /// Notify about a new relative pointer motion event.
    /// The (dx, dy) vector represents the new pointer position in the streams logical coordinate space.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - The [`Session`] object handle
    /// * `options` - ?
    /// * `dx` - Relative movement on the x axis
    /// * `dy` - Relative movement on the y axis
    ///
    /// [`Session`]: ../session/struct.SessionProxy.html
    fn notify_pointer_motion(
        &self,
        session_handle: ObjectPath,
        options: HashMap<&str, Value>,
        dx: f64,
        dy: f64,
    ) -> Result<()>;

    /// Notify about a new absolute pointer motion event.
    /// The (x, y) position represents the new pointer position in the streams logical coordinate sspace
    ///
    /// # Arguments
    ///
    /// * `session_handle` - The [`Session`] object handle
    /// * `options` - ?
    /// * `stream` - The PipeWire stream node the coordinate is relative to
    /// * `x` - Pointer motion x coordinate
    /// * `y` - Pointer motion y coordinate
    ///
    /// [`Session`]: ../session/struct.SessionProxy.html
    fn notify_pointer_motion_absolute(
        &self,
        session_handle: ObjectPath,
        options: HashMap<&str, Value>,
        stream: u32,
        x: f64,
        y: f64,
    ) -> Result<()>;

    /// Notify about a new touch down event.
    /// The (x, y) position represents the new touch point position in the streams logical coordinate space
    ///
    /// May only be called if TOUCHSCREEN access was provided after starting the session.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - The [`Session`] object handle
    /// * `options` - ?
    /// * `stream` - The PipeWire stream node the coordinate is relative to
    /// * `slot` - Touch slot where touch point appeared
    /// * `x` - Touch down x coordinate
    /// * `y` - Touch down y coordinate
    ///
    /// [`Session`]: ../session/struct.SessionProxy.html
    fn notify_touch_down(
        &self,
        session_handle: ObjectPath,
        options: HashMap<&str, Value>,
        stream: u32,
        slot: u32,
        x: f64,
        y: f64,
    ) -> Result<()>;

    /// Notify about a new touch motion event.
    /// The (x, y) position represents where the touch point position in the streams logical coordinate space moved
    ///
    /// May only be called if TOUCHSCREEN access was provided after starting the session.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - The [`Session`] object handle
    /// * `options` - ?
    /// * `stream` - The PipeWire stream node the coordinate is relative to
    /// * `slot` - Touch slot where touch point appeared
    /// * `x` - Touch motion x coordinate
    /// * `y` - Touch motion y coordinate
    ///
    /// [`Session`]: ../session/struct.SessionProxy.html
    fn notify_touch_motion(
        &self,
        session_handle: ObjectPath,
        options: HashMap<&str, Value>,
        stream: u32,
        slot: u32,
        x: f64,
        y: f64,
    ) -> Result<()>;

    /// Notify about a new touch up event.
    ///
    /// May only be called if TOUCHSCREEN access was provided after starting the session.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - The [`Session`] object handle
    /// * `options` - ?
    /// * `slot` - Touch slot where touch point appeared
    ///
    /// [`Session`]: ../session/struct.SessionProxy.html
    fn notify_touch_up(
        &self,
        session_handle: ObjectPath,
        options: HashMap<&str, Value>,
        slot: u32,
    ) -> Result<()>;

    /// Select input devices to remote control.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - The [`Session`] object handle
    /// * `options` - [`SelectDevicesOptions`]
    ///
    /// [`SelectDevicesOptions`]: ../struct.SelectDevicesOptions.html
    /// [`Session`]: ../session/struct.SessionProxy.html
    fn select_devices(
        &self,
        session_handle: ObjectPath,
        options: SelectDevicesOptions,
    ) -> Result<OwnedObjectPath>;

    ///  Start the remote desktop session.
    ///
    /// This will typically result in the portal presenting a dialog letting
    /// the user select what to share, including devices and optionally screen content
    /// if screen cast sources was selected.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - The [`Session`] object handle
    /// * `parent_window` - The application window identifier
    /// * `options` - [`StartRemoteOptions`]
    ///
    /// [`StartRemoteOptions`]: ../struct.StartRemoteOptions.html
    /// [`Session`]: ../session/struct.SessionProxy.html
    fn start(
        &self,
        session_handle: ObjectPath,
        parent_window: WindowIdentifier,
        options: StartRemoteOptions,
    ) -> Result<OwnedObjectPath>;

    /// Available source types.
    #[dbus_proxy(property)]
    fn available_device_types(&self) -> Result<u32>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

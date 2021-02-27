//! # Examples
//!
//! ```rust,no_run
//! use ashpd::desktop::remote_desktop::{
//!     CreateRemoteOptions, CreateSession, DeviceType, KeyState, RemoteDesktopProxy,
//!     SelectDevicesOptions, SelectedDevices, StartRemoteOptions,
//! };
//! use ashpd::{BasicResponse as Basic, HandleToken, RequestProxy, Response, WindowIdentifier};
//! use std::collections::HashMap;
//! use std::convert::TryFrom;
//! use zbus::{fdo::Result, Connection};
//! use zvariant::ObjectPath;
//!
//! fn select_devices(
//!     handle: &'static ObjectPath<'_>,
//!     connection: &'static Connection,
//!     proxy: &'static RemoteDesktopProxy,
//! ) -> Result<()> {
//!     let request = proxy.select_devices(handle,
//!         SelectDevicesOptions::default().types(DeviceType::Keyboard | DeviceType::Pointer),
//!     )?;
//!
//!     request.connect_response(move |r: Response<Basic>| {
//!         if r.is_ok() {
//!             start_remote(handle, connection, proxy)?;
//!         }
//!         Ok(())
//!     })?;
//!
//!     Ok(())
//! }
//!
//! fn start_remote(
//!     handle: &'static ObjectPath<'_>,
//!     connection: &'static Connection,
//!     proxy: &'static RemoteDesktopProxy,
//! ) -> Result<()> {
//!     let request = proxy.start(
//!         handle,
//!         WindowIdentifier::default(),
//!         StartRemoteOptions::default(),
//!     )?;
//!
//!     request.connect_response(move |r: Response<SelectedDevices>| {
//!         proxy.notify_keyboard_keycode(
//!             handle,
//!             HashMap::new(),
//!             13, // Enter key code
//!             KeyState::Pressed,
//!         )?;
//!
//!         println!("{:#?}", r.unwrap().devices);
//!         Ok(())
//!     })?;
//!
//!     Ok(())
//! }
//!
//! fn main() -> Result<()> {
//!     let connection = Connection::new_session()?;
//!     let proxy = RemoteDesktopProxy::new(&connection)?;
//!
//!     let request = proxy.create_session(
//!         CreateRemoteOptions::default()
//!             .session_handle_token(HandleToken::try_from("token").unwrap()),
//!     )?;
//!
//!     request.connect_response(move |r: Response<CreateSession>| {
//!         let session = r.unwrap();
//!         select_devices(session.handle(), &connection, &proxy)?;
//!         Ok(())
//!     })?;
//!
//!     Ok(())
//! }
//! ```
use std::collections::HashMap;

use enumflags2::BitFlags;
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{ObjectPath, OwnedObjectPath, Value};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

use crate::{AsyncRequestProxy, HandleToken, RequestProxy, WindowIdentifier};

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
/// Specified options on a create a remote session request.
pub struct CreateRemoteOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
    /// A string that will be used as the last element of the session handle.
    session_handle_token: Option<HandleToken>,
}

impl CreateRemoteOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    /// Sets the session handle token.
    pub fn session_handle_token(mut self, session_handle_token: HandleToken) -> Self {
        self.session_handle_token = Some(session_handle_token);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// A response to a create session request.
pub struct CreateSession {
    /// A string that will be used as the last element of the session handle.
    session_handle: OwnedObjectPath,
}

impl CreateSession {
    /// The created session handle.
    pub fn handle(&self) -> &ObjectPath<'_> {
        &self.session_handle
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a select devices request.
pub struct SelectDevicesOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
    /// The device types to request remote controlling of. Default is all.
    types: Option<BitFlags<DeviceType>>,
}

impl SelectDevicesOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }

    /// Sets the device types to request remote controlling of.
    pub fn types(mut self, types: BitFlags<DeviceType>) -> Self {
        self.types = Some(types);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a start remote desktop request.
pub struct StartRemoteOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: Option<HandleToken>,
}

impl StartRemoteOptions {
    /// Sets the handle token.
    pub fn handle_token(mut self, handle_token: HandleToken) -> Self {
        self.handle_token = Some(handle_token);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// A response to a select device request.
pub struct SelectedDevices {
    /// The selected devices.
    pub devices: BitFlags<DeviceType>,
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.RemoteDesktop",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications create remote desktop sessions.
pub trait RemoteDesktop {
    /// Create a remote desktop session.
    /// A remote desktop session is used to allow remote controlling a desktop
    /// session. It can also be used together with a screen cast session.
    ///
    /// # Arguments
    ///
    /// * `options` - A [`CreateRemoteOptions`].
    ///
    /// [`CreateRemoteOptions`]: ./struct.CreateRemoteOptions.html
    #[dbus_proxy(object = "Request")]
    fn create_session(&self, options: CreateRemoteOptions);

    /// Notify keyboard code.
    /// May only be called if KEYBOARD access was provided after starting the
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `options` - ?
    /// * `keycode` - Keyboard code that was pressed or released.
    /// * `state` - The new state of the keyboard code.
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    ///
    /// FIXME: figure out the options we can take here
    fn notify_keyboard_keycode(
        &self,
        session_handle: &ObjectPath<'_>,
        options: HashMap<&str, Value<'_>>,
        keycode: i32,
        state: KeyState,
    ) -> Result<()>;

    /// Notify keyboard symbol.
    /// May only be called if KEYBOARD access was provided after starting the
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `options` - ?
    /// * `keysym` - Keyboard symbol that was pressed or released.
    /// * `state` - The new state of the keyboard code.
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    ///
    /// FIXME: figure out the options we can take here
    fn notify_keyboard_keysym(
        &self,
        session_handle: &ObjectPath<'_>,
        options: HashMap<&str, Value<'_>>,
        keysym: i32,
        state: KeyState,
    ) -> Result<()>;

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
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `options` - ?
    /// * `dx` - Relative axis movement on the x axis.
    /// * `dy` - Relative axis movement on the y axis.
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    ///
    /// FIXME: figure out the options we can take here
    fn notify_pointer_axis(
        &self,
        session_handle: &ObjectPath<'_>,
        options: HashMap<&str, Value<'_>>,
        dx: f64,
        dy: f64,
    ) -> Result<()>;

    /// Notify pointer axis discrete.
    /// May only be called if POINTER access was provided after starting the
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `options` - ?
    /// * `axis` - The axis that was scrolled.
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    ///
    /// FIXME: figure out the options we can take here
    fn notify_pointer_axis_discrete(
        &self,
        session_handle: &ObjectPath<'_>,
        options: HashMap<&str, Value<'_>>,
        axis: Axis,
        steps: i32,
    ) -> Result<()>;

    /// Notify pointer button.
    /// The pointer button is encoded according to Linux Evdev button codes.
    ///
    ///  May only be called if POINTER access was provided after starting the
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `options` - ?
    /// * `button` - The pointer button was pressed or released.
    /// * `state` - The new state of the keyboard code.
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    ///
    /// FIXME: figure out the options we can take here
    fn notify_pointer_button(
        &self,
        session_handle: &ObjectPath<'_>,
        options: HashMap<&str, Value<'_>>,
        button: i32,
        state: KeyState,
    ) -> Result<()>;

    /// Notify about a new relative pointer motion event.
    /// The (dx, dy) vector represents the new pointer position in the streams
    /// logical coordinate space.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `options` - ?
    /// * `dx` - Relative movement on the x axis.
    /// * `dy` - Relative movement on the y axis.
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    ///
    /// FIXME: figure out the options we can take here
    fn notify_pointer_motion(
        &self,
        session_handle: &ObjectPath<'_>,
        options: HashMap<&str, Value<'_>>,
        dx: f64,
        dy: f64,
    ) -> Result<()>;

    /// Notify about a new absolute pointer motion event.
    /// The (x, y) position represents the new pointer position in the streams
    /// logical coordinate space.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `options` - ?
    /// * `stream` - The PipeWire stream node the coordinate is relative to.
    /// * `x` - Pointer motion x coordinate.
    /// * `y` - Pointer motion y coordinate.
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    ///
    /// FIXME: figure out the options we can take here
    fn notify_pointer_motion_absolute(
        &self,
        session_handle: &ObjectPath<'_>,
        options: HashMap<&str, Value<'_>>,
        stream: u32,
        x: f64,
        y: f64,
    ) -> Result<()>;

    /// Notify about a new touch down event.
    /// The (x, y) position represents the new touch point position in the
    /// streams logical coordinate space.
    ///
    /// May only be called if TOUCHSCREEN access was provided after starting the
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `options` - ?
    /// * `stream` - The PipeWire stream node the coordinate is relative to.
    /// * `slot` - Touch slot where touch point appeared.
    /// * `x` - Touch down x coordinate.
    /// * `y` - Touch down y coordinate.
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    ///
    /// FIXME: figure out the options we can take here
    fn notify_touch_down(
        &self,
        session_handle: &ObjectPath<'_>,
        options: HashMap<&str, Value<'_>>,
        stream: u32,
        slot: u32,
        x: f64,
        y: f64,
    ) -> Result<()>;

    /// Notify about a new touch motion event.
    /// The (x, y) position represents where the touch point position in the
    /// streams logical coordinate space moved.
    ///
    /// May only be called if TOUCHSCREEN access was provided after starting the
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `options` - ?
    /// * `stream` - The PipeWire stream node the coordinate is relative to.
    /// * `slot` - Touch slot where touch point appeared.
    /// * `x` - Touch motion x coordinate.
    /// * `y` - Touch motion y coordinate.
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    ///
    /// FIXME: figure out the options we can take here
    fn notify_touch_motion(
        &self,
        session_handle: &ObjectPath<'_>,
        options: HashMap<&str, Value<'_>>,
        stream: u32,
        slot: u32,
        x: f64,
        y: f64,
    ) -> Result<()>;

    /// Notify about a new touch up event.
    ///
    /// May only be called if TOUCHSCREEN access was provided after starting the
    /// session.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `options` - ?
    /// * `slot` - Touch slot where touch point appeared.
    ///
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    ///
    /// FIXME: figure out the options we can take here
    fn notify_touch_up(
        &self,
        session_handle: &ObjectPath<'_>,
        options: HashMap<&str, Value<'_>>,
        slot: u32,
    ) -> Result<()>;

    /// Select input devices to remote control.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `options` - [`SelectDevicesOptions`].
    ///
    /// [`SelectDevicesOptions`]: ../struct.SelectDevicesOptions.html
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    #[dbus_proxy(object = "Request")]
    fn select_devices(&self, session_handle: &ObjectPath<'_>, options: SelectDevicesOptions);

    ///  Start the remote desktop session.
    ///
    /// This will typically result in the portal presenting a dialog letting
    /// the user select what to share, including devices and optionally screen
    /// content if screen cast sources was selected.
    ///
    /// # Arguments
    ///
    /// * `session_handle` - A [`SessionProxy`] object path.
    /// * `parent_window` - The application window identifier.
    /// * `options` - [`StartRemoteOptions`].
    ///
    /// [`StartRemoteOptions`]: ../struct.StartRemoteOptions.html
    /// [`SessionProxy`]: ../../session/struct.SessionProxy.html
    #[dbus_proxy(object = "Request")]
    fn start(
        &self,
        session_handle: &ObjectPath<'_>,
        parent_window: WindowIdentifier,
        options: StartRemoteOptions,
    );

    /// Available source types.
    #[dbus_proxy(property)]
    fn available_device_types(&self) -> Result<u32>;

    /// The version of this DBus interface.
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

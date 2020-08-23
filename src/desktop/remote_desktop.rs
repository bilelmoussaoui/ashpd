use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.RemoteDesktop",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications create remote desktop sessions.
trait RemoteDesktop {
    /// CreateSession method
    fn create_session(&self, options: HashMap<&str, zvariant::Value>) -> Result<String>;

    /// NotifyKeyboardKeycode method
    fn notify_keyboard_keycode(
        &self,
        session_handle: &str,
        options: HashMap<&str, zvariant::Value>,
        keycode: i32,
        state: u32,
    ) -> Result<()>;

    /// NotifyKeyboardKeysym method
    fn notify_keyboard_keysym(
        &self,
        session_handle: &str,
        options: HashMap<&str, zvariant::Value>,
        keysym: i32,
        state: u32,
    ) -> Result<()>;

    /// NotifyPointerAxis method
    fn notify_pointer_axis(
        &self,
        session_handle: &str,
        options: HashMap<&str, zvariant::Value>,
        dx: f64,
        dy: f64,
    ) -> Result<()>;

    /// NotifyPointerAxisDiscrete method
    fn notify_pointer_axis_discrete(
        &self,
        session_handle: &str,
        options: HashMap<&str, zvariant::Value>,
        axis: u32,
        steps: i32,
    ) -> Result<()>;

    /// NotifyPointerButton method
    fn notify_pointer_button(
        &self,
        session_handle: &str,
        options: HashMap<&str, zvariant::Value>,
        button: i32,
        state: u32,
    ) -> Result<()>;

    /// NotifyPointerMotion method
    fn notify_pointer_motion(
        &self,
        session_handle: &str,
        options: HashMap<&str, zvariant::Value>,
        dx: f64,
        dy: f64,
    ) -> Result<()>;

    /// NotifyPointerMotionAbsolute method
    fn notify_pointer_motion_absolute(
        &self,
        session_handle: &str,
        options: HashMap<&str, zvariant::Value>,
        stream: u32,
        x: f64,
        y: f64,
    ) -> Result<()>;

    /// NotifyTouchDown method
    fn notify_touch_down(
        &self,
        session_handle: &str,
        options: HashMap<&str, zvariant::Value>,
        stream: u32,
        slot: u32,
        x: f64,
        y: f64,
    ) -> Result<()>;

    /// NotifyTouchMotion method
    fn notify_touch_motion(
        &self,
        session_handle: &str,
        options: HashMap<&str, zvariant::Value>,
        stream: u32,
        slot: u32,
        x: f64,
        y: f64,
    ) -> Result<()>;

    /// NotifyTouchUp method
    fn notify_touch_up(
        &self,
        session_handle: &str,
        options: HashMap<&str, zvariant::Value>,
        slot: u32,
    ) -> Result<()>;

    /// SelectDevices method
    fn select_devices(
        &self,
        session_handle: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// Start method
    fn start(
        &self,
        session_handle: &str,
        parent_window: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// AvailableDeviceTypes property
    #[dbus_proxy(property)]
    fn available_device_types(&self) -> Result<u32>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

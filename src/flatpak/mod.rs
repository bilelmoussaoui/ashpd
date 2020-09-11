use enumflags2::BitFlags;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Copy, Clone, BitFlags, Debug, Type)]
#[repr(u32)]
pub enum SandboxFlags {
    /// Share the display access (X11, wayland) with the caller.
    DisplayAccess = 1,
    /// Share the sound access (pulseaudio) with the caller.
    SoundAccess = 2,
    /// Share the gpu access with the caller.
    GPUAccess = 4,
    /// Allow sandbox access to (filtered) session bus.
    SessionBusAccess = 8,
    /// Allow sandbox access to accessibility bus.
    AccessibilityBusAccess = 16,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Copy, Clone, BitFlags, Debug, Type)]
#[repr(u32)]
pub enum SupportsFlags {
    /// Supports the expose sandbox pids flag of Spawn.
    ExposePids = 1,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
pub struct SpawnOptions {
    /// A list of filenames for files inside the sandbox that will be exposed to the new sandbox, for reading and writing.
    /// Note that absolute paths or subdirectories are not allowed.
    pub sandbox_expose: Option<Vec<String>>,
    /// A list of filenames for files inside the sandbox that will be exposed to the new sandbox, readonly.
    /// Note that absolute paths or subdirectories are not allowed.
    pub sandbox_expose_ro: Option<Vec<String>>,
    /// A list of file descriptor for files inside the sandbox that will be exposed to the new sandbox, for reading and writing.
    pub sandbox_expose_fd: Option<Vec<RawFd>>,
    /// A list of file descriptor for files inside the sandbox that will be exposed to the new sandbox, readonly.
    pub sandbox_expose_fd_ro: Option<Vec<RawFd>>,
    /// Flags affecting the created sandbox.
    pub sandbox_flags: Option<BitFlags<SandboxFlags>>,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
pub struct CreateMonitorOptions {}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Flatpak",
    default_service = "org.freedesktop.portal.Flatpak",
    default_path = "/org/freedesktop/portal/Flatpak"
)]
/// The interface exposes some interactions with Flatpak on the host to the sandbox.
/// For example, it allows you to restart the applications or start a more sandboxed instance.
trait Flatpak {
    /// Creates an update monitor object that will emit signals
    /// when an update for the caller becomes available, and can be used to install it.
    fn create_update_monitor(&self, options: CreateMonitorOptions) -> Result<String>;

    /// This methods let you start a new instance of your application, optionally enabling a tighter sandbox.
    ///
    /// Returns the PID of the new process
    ///
    /// # Arguments
    ///
    /// * `cwd_path` - the working directory for the new process
    /// * `arvg` - the argv for the new process, starting with the executable to launch
    /// * `fds` - Array of file descriptors to pass to the new process
    /// * `envs` - Array of variable/value pairs for the environment of the new process
    /// * `flags`
    /// * `options` - A [`SpawnOptions`]
    ///
    /// [`SpawnOptions`]: ./struct.SpawnOptions.html
    fn spawn(
        &self,
        cwd_path: &[u8],
        argv: &[&[u8]],
        fds: HashMap<u32, RawFd>,
        envs: HashMap<&str, &str>,
        flags: BitFlags<SandboxFlags>,
        options: SpawnOptions,
    ) -> Result<u32>;

    /// This methods let you send a Unix signal to a process that was started `spawn`
    ///
    /// # Arguments
    ///
    /// * `pid` - the PID of the process to send the signal to
    /// * `signal` - the signal to send
    /// * `to_process_group` - whether to send the signal to the process group
    fn spawn_signal(&self, pid: u32, signal: u32, to_process_group: bool) -> Result<()>;

    // signal
    // fn spawn_started(&self, pid: u32, relpid: u32);

    // signal
    // fn spawn_existed(&self, pid: u32, exit_status: u32);

    /// Flags marking what optional features are available.
    #[dbus_proxy(property)]
    fn supports(&self) -> Result<u32>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

pub mod update_monitor;

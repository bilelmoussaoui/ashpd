//! # Examples
//!
//! Spawn a process outside of the sandbox, only works in a Flatpak.
//!
//! ```no_run
//! use enumflags2::BitFlags;
//! use ashpd::flatpak::{FlatpakProxy, SpawnFlags, SpawnOptions};
//! use zbus::{fdo::Result, Connection};
//! use std::collections::HashMap;
//!
//! fn main() -> Result<()> {
//!     let connection = Connection::new_session()?;
//!     let proxy = FlatpakProxy::new(&connection)?;
//!
//!     proxy.spawn(
//!         "contrast".into(),
//!         vec!["".into()],
//!         HashMap::new(),
//!         HashMap::new(),
//!         SpawnFlags::ClearEnv | SpawnFlags::NoNetwork,
//!         SpawnOptions::default(),
//!     )?;
//!
//!     Ok(())
//! }
//! ```
use crate::NString;
use enumflags2::BitFlags;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{Fd, OwnedObjectPath};
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
/// Flags affecting the created sandbox.
pub enum SpawnFlags {
    /// Clear the environment.
    ClearEnv = 1,
    /// Spawn the latest version of the app.
    Latest = 2,
    /// Spawn in a sandbox (equivalent of the sandbox option of flatpak run).
    Sandbox = 4,
    /// Spawn without network (equivalent of the unshare=network option of flatpak run).
    NoNetwork = 8,
    /// Kill the sandbox when the caller disappears from the session bus.
    Kill = 16,
    /// Expose the sandbox pids in the callers sandbox, only supported if using user namespaces for containers (not setuid), see the support property.
    Expose = 32,
    /// Emit a SpawnStarted signal once the sandboxed process has been fully started.
    Emit = 64,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Copy, Clone, BitFlags, Debug, Type)]
#[repr(u32)]
/// Flags marking what optional features are available.
pub enum SupportsFlags {
    /// Supports the expose sandbox pids flag of Spawn.
    ExposePids = 1,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specficied options on a spawn request.
pub struct SpawnOptions {
    /// A list of filenames for files inside the sandbox that will be exposed to the new sandbox, for reading and writing.
    /// Note that absolute paths or subdirectories are not allowed.
    pub sandbox_expose: Option<Vec<String>>,
    /// A list of filenames for files inside the sandbox that will be exposed to the new sandbox, readonly.
    /// Note that absolute paths or subdirectories are not allowed.
    pub sandbox_expose_ro: Option<Vec<String>>,
    /// A list of file descriptor for files inside the sandbox that will be exposed to the new sandbox, for reading and writing.
    pub sandbox_expose_fd: Option<Vec<Fd>>,
    /// A list of file descriptor for files inside the sandbox that will be exposed to the new sandbox, readonly.
    pub sandbox_expose_fd_ro: Option<Vec<Fd>>,
    /// Flags affecting the created sandbox.
    pub sandbox_flags: Option<BitFlags<SandboxFlags>>,
}

impl SpawnOptions {
    /// Sets the list of filenames for files to expose the new sandbox.
    pub fn sandbox_expose(mut self, sandbox_expose: Vec<String>) -> Self {
        self.sandbox_expose = Some(sandbox_expose);
        self
    }

    /// Sets the list of filenames for files to expose the new sandbox, readonly.
    pub fn sandbox_expose_ro(mut self, sandbox_expose_ro: Vec<String>) -> Self {
        self.sandbox_expose_ro = Some(sandbox_expose_ro);
        self
    }

    /// Sets the list of file descriptors of files to expose the new sandbox.
    pub fn sandbox_expose_fd(mut self, sandbox_expose_fd: Vec<Fd>) -> Self {
        self.sandbox_expose_fd = Some(sandbox_expose_fd);
        self
    }

    /// Sets the list of file descriptors of files to expose the new sandbox, readonly.
    pub fn sandbox_expose_fd_ro(mut self, sandbox_expose_fd_ro: Vec<Fd>) -> Self {
        self.sandbox_expose_fd_ro = Some(sandbox_expose_fd_ro);
        self
    }

    /// Sets the created sandbox flags.
    pub fn sandbox_flags(mut self, sandbox_flags: BitFlags<SandboxFlags>) -> Self {
        self.sandbox_flags = Some(sandbox_flags);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specficied options on a create monitor request.
///
/// Currently there are no possible options yet.
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
    fn create_update_monitor(&self, options: CreateMonitorOptions) -> Result<OwnedObjectPath>;

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
        cwd_path: NString,
        argv: Vec<NString>,
        fds: HashMap<u32, Fd>,
        envs: HashMap<&str, &str>,
        flags: BitFlags<SpawnFlags>,
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

    // FIXME: signal
    // fn spawn_started(&self, pid: u32, relpid: u32);

    // FIXME: signal
    // fn spawn_existed(&self, pid: u32, exit_status: u32);

    /// Flags marking what optional features are available.
    #[dbus_proxy(property)]
    fn supports(&self) -> Result<u32>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

pub mod update_monitor;

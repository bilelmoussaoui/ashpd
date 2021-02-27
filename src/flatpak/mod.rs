//! # Examples
//!
//! Spawn a process outside of the sandbox, only works in a Flatpak.
//!
//! ```rust,no_run
//! use ashpd::flatpak::{FlatpakProxy, SpawnFlags, SpawnOptions};
//! use enumflags2::BitFlags;
//! use std::collections::HashMap;
//! use zbus::{fdo::Result, Connection};
//!
//! fn main() -> Result<()> {
//!     let connection = Connection::new_session()?;
//!     let proxy = FlatpakProxy::new(&connection)?;
//!
//!     proxy.spawn(
//!         "contrast".into(),
//!         &[],
//!         HashMap::new(),
//!         HashMap::new(),
//!         SpawnFlags::ClearEnv | SpawnFlags::NoNetwork,
//!         SpawnOptions::default(),
//!     )?;
//!
//!     Ok(())
//! }
//! ```
use std::collections::HashMap;

use enumflags2::BitFlags;
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::Fd;
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

use crate::flatpak::update_monitor::{AsyncUpdateMonitorProxy, UpdateMonitorProxy};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Copy, Clone, BitFlags, Debug, Type)]
#[repr(u32)]
/// A bitmask representing the "permissions" of a newly created sandbox.
pub enum SandboxFlags {
    /// Share the display access (X11, Wayland) with the caller.
    DisplayAccess = 1,
    /// Share the sound access (PulseAudio) with the caller.
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
    /// Spawn in a sandbox (equivalent of the sandbox option of `flatpak run`).
    Sandbox = 4,
    /// Spawn without network (equivalent of the `unshare=network` option of
    /// `flatpak run`).
    NoNetwork = 8,
    /// Kill the sandbox when the caller disappears from the session bus.
    Kill = 16,
    /// Expose the sandbox pids in the callers sandbox, only supported if using
    /// user namespaces for containers (not setuid), see the support property.
    Expose = 32,
    /// Emit a SpawnStarted signal once the sandboxed process has been fully
    /// started.
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
/// Specified options on a spawn request.
pub struct SpawnOptions {
    /// A list of filenames for files inside the sandbox that will be exposed to
    /// the new sandbox, for reading and writing.
    sandbox_expose: Option<Vec<String>>,
    /// A list of filenames for files inside the sandbox that will be exposed to
    /// the new sandbox, read-only.
    sandbox_expose_ro: Option<Vec<String>>,
    /// A list of file descriptor for files inside the sandbox that will be
    /// exposed to the new sandbox, for reading and writing.
    sandbox_expose_fd: Option<Vec<Fd>>,
    /// A list of file descriptor for files inside the sandbox that will be
    /// exposed to the new sandbox, read-only.
    sandbox_expose_fd_ro: Option<Vec<Fd>>,
    /// Flags affecting the created sandbox.
    sandbox_flags: Option<BitFlags<SandboxFlags>>,
}

impl SpawnOptions {
    /// Sets the list of filenames for files to expose the new sandbox.
    /// **Note** that absolute paths or subdirectories are not allowed.
    pub fn sandbox_expose(mut self, sandbox_expose: &[&str]) -> Self {
        self.sandbox_expose = Some(
            sandbox_expose
                .to_vec()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );
        self
    }

    /// Sets the list of filenames for files to expose the new sandbox,
    /// read-only. **Note** that absolute paths or subdirectories are not
    /// allowed.
    pub fn sandbox_expose_ro(mut self, sandbox_expose_ro: &[&str]) -> Self {
        self.sandbox_expose_ro = Some(
            sandbox_expose_ro
                .to_vec()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );
        self
    }

    /// Sets the list of file descriptors of files to expose the new sandbox.
    pub fn sandbox_expose_fd(mut self, sandbox_expose_fd: &[Fd]) -> Self {
        self.sandbox_expose_fd = Some(sandbox_expose_fd.to_vec());
        self
    }

    /// Sets the list of file descriptors of files to expose the new sandbox,
    /// read-only.
    pub fn sandbox_expose_fd_ro(mut self, sandbox_expose_fd_ro: &[Fd]) -> Self {
        self.sandbox_expose_fd_ro = Some(sandbox_expose_fd_ro.to_vec());
        self
    }

    /// Sets the created sandbox flags.
    pub fn sandbox_flags(mut self, sandbox_flags: BitFlags<SandboxFlags>) -> Self {
        self.sandbox_flags = Some(sandbox_flags);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options on a create monitor request.
///
/// Currently there are no possible options yet.
pub struct CreateMonitorOptions {}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Flatpak",
    default_service = "org.freedesktop.portal.Flatpak",
    default_path = "/org/freedesktop/portal/Flatpak"
)]
/// The interface exposes some interactions with Flatpak on the host to the
/// sandbox. For example, it allows you to restart the applications or start a
/// more sandboxed instance.
trait Flatpak {
    /// Creates an update monitor object that will emit signals
    /// when an update for the caller becomes available, and can be used to
    /// install it.
    #[dbus_proxy(object = "UpdateMonitor")]
    fn create_update_monitor(&self, options: CreateMonitorOptions);

    /// This methods let you start a new instance of your application,
    /// optionally enabling a tighter sandbox.
    ///
    /// Returns the PID of the new process.
    ///
    /// # Arguments
    ///
    /// * `cwd_path` - The working directory for the new process.
    /// * `arvg` - The argv for the new process, starting with the executable to
    ///   launch.
    /// * `fds` - Array of file descriptors to pass to the new process.
    /// * `envs` - Array of variable/value pairs for the environment of the new
    ///   process.
    /// * `flags`
    /// * `options` - A [`SpawnOptions`].
    ///
    /// [`SpawnOptions`]: ./struct.SpawnOptions.html
    fn spawn(
        &self,
        cwd_path: &str,
        argv: &[&str],
        fds: HashMap<u32, Fd>,
        envs: HashMap<&str, &str>,
        flags: BitFlags<SpawnFlags>,
        options: SpawnOptions,
    ) -> Result<u32>;

    /// This methods let you send a Unix signal to a process that was started
    /// `spawn`.
    ///
    /// # Arguments
    ///
    /// * `pid` - The PID of the process to send the signal to.
    /// * `signal` - The signal to send.
    /// * `to_process_group` - Whether to send the signal to the process group.
    fn spawn_signal(&self, pid: u32, signal: u32, to_process_group: bool) -> Result<()>;

    #[dbus_proxy(signal)]
    fn spawn_started(&self, pid: u32, relpid: u32) -> Result<()>;

    #[dbus_proxy(signal)]
    fn spawn_existed(&self, pid: u32, exit_status: u32) -> Result<()>;

    /// Flags marking what optional features are available.
    #[dbus_proxy(property)]
    fn supports(&self) -> Result<u32>;

    /// The version of this DBus interface.
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

/// Monitor if there's an update it and install it.
pub mod update_monitor;

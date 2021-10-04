//! # Examples
//!
//! Spawn a process outside of the sandbox, only works in a Flatpak.
//!
//! ```rust,no_run
//! use ashpd::flatpak::{FlatpakProxy, SpawnFlags, SpawnOptions};
//! use enumflags2::BitFlags;
//! use std::collections::HashMap;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let connection = zbus::Connection::session().await?;
//!     let proxy = FlatpakProxy::new(&connection).await?;
//!
//!     proxy
//!         .spawn(
//!             "/",
//!             &["contrast"],
//!             HashMap::new(),
//!             HashMap::new(),
//!             SpawnFlags::ClearEnv | SpawnFlags::NoNetwork,
//!             SpawnOptions::default(),
//!         )
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

pub(crate) const DESTINATION: &str = "org.freedesktop.portal.Flatpak";
pub(crate) const PATH: &str = "/org/freedesktop/portal/Flatpak";

use enumflags2::BitFlags;
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::os::unix::ffi::OsStrExt;
use std::{collections::HashMap, ffi::CString, fmt::Debug, os::unix::prelude::AsRawFd, path::Path};
use zvariant::Fd;
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

use crate::{
    helpers::{call_method, receive_signal},
    Error,
};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Copy, Clone, BitFlags, Debug, Type)]
#[repr(u32)]
/// A bitmask representing the "permissions" of a newly created sandbox.
pub enum SandboxFlags {
    /// Share the display access (X11, Wayland) with the caller.
    DisplayAccess = 1,
    /// Share the sound access (PulseAudio) with the caller.
    SoundAccess = 2,
    /// Share the gpu access with the caller.
    GpuAccess = 4,
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
/// Specified options for a [`FlatpakProxy::spawn`] request.
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
    /// **Note** absolute paths or subdirectories are not allowed.
    pub fn sandbox_expose<S: AsRef<str> + zvariant::Type + Serialize>(
        mut self,
        sandbox_expose: &[S],
    ) -> Self {
        self.sandbox_expose = Some(
            sandbox_expose
                .iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
        );
        self
    }

    /// Sets the list of filenames for files to expose the new sandbox,
    /// read-only.
    /// **Note** absolute paths or subdirectories are not allowed.
    pub fn sandbox_expose_ro<S: AsRef<str> + zvariant::Type + Serialize>(
        mut self,
        sandbox_expose_ro: &[S],
    ) -> Self {
        self.sandbox_expose_ro = Some(
            sandbox_expose_ro
                .iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
        );
        self
    }

    /// Sets the list of file descriptors of files to expose the new sandbox.
    pub fn sandbox_expose_fd<F: AsRawFd>(mut self, sandbox_expose_fd: &[&F]) -> Self {
        self.sandbox_expose_fd = Some(
            sandbox_expose_fd
                .iter()
                .map(|f| Fd::from(f.as_raw_fd()))
                .collect(),
        );
        self
    }

    /// Sets the list of file descriptors of files to expose the new sandbox,
    /// read-only.
    pub fn sandbox_expose_fd_ro<F: AsRawFd>(mut self, sandbox_expose_fd_ro: &[&F]) -> Self {
        self.sandbox_expose_fd_ro = Some(
            sandbox_expose_fd_ro
                .iter()
                .map(|f| Fd::from(f.as_raw_fd()))
                .collect(),
        );
        self
    }

    /// Sets the created sandbox flags.
    pub fn sandbox_flags(mut self, sandbox_flags: BitFlags<SandboxFlags>) -> Self {
        self.sandbox_flags = Some(sandbox_flags);
        self
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a [`FlatpakProxy::create_update_monitor`] request.
///
/// Currently there are no possible options yet.
struct CreateMonitorOptions {}

/// The interface exposes some interactions with Flatpak on the host to the
/// sandbox. For example, it allows you to restart the applications or start a
/// more sandboxed instance.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Flatpak`](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html#gdbus-org.freedesktop.portal.Flatpak).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Flatpak")]
pub struct FlatpakProxy<'a>(zbus::Proxy<'a>);

impl<'a> FlatpakProxy<'a> {
    /// Create a new instance of [`FlatpakProxy`].
    pub async fn new(connection: &zbus::Connection) -> Result<FlatpakProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Flatpak")?
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

    /// Creates an update monitor object that will emit signals
    /// when an update for the caller becomes available, and can be used to
    /// install it.
    ///
    /// # Specifications
    ///
    /// See also [`CreateUpdateMonitor`](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html#gdbus-method-org-freedesktop-portal-Flatpak.CreateUpdateMonitor).
    #[doc(alias = "CreateUpdateMonitor")]
    pub async fn create_update_monitor(&self) -> Result<UpdateMonitorProxy<'a>, Error> {
        let options = CreateMonitorOptions::default();
        let path: zvariant::OwnedObjectPath =
            call_method(&self.0, "CreateUpdateMonitor", &(options)).await?;

        UpdateMonitorProxy::new(self.0.connection(), path.into_inner()).await
    }

    /// Emitted when a process starts by [`spawn()`][`FlatpakProxy::spawn`].
    pub async fn receive_spawn_started(&self) -> Result<(u32, u32), Error> {
        receive_signal(&self.0, "SpawnStarted").await
    }

    /// Emitted when a process started by [`spawn()`][`FlatpakProxy::spawn`]
    /// exits.
    ///
    /// # Specifications
    ///
    /// See also [`SpawnExited`](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html#gdbus-signal-org-freedesktop-portal-Flatpak.SpawnExited).
    #[doc(alias = "SpawnExited")]
    pub async fn receive_spawn_existed(&self) -> Result<(u32, u32), Error> {
        receive_signal(&self.0, "SpawnExited").await
    }

    /// This methods let you start a new instance of your application,
    /// optionally enabling a tighter sandbox.
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
    /// # Returns
    ///
    /// The PID of the new process.
    ///
    /// # Specifications
    ///
    /// See also [`Spawn`](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html#gdbus-method-org-freedesktop-portal-Flatpak.Spawn).
    #[doc(alias = "Spawn")]
    pub async fn spawn<
        C: AsRef<Path> + zvariant::Type + Serialize + Debug,
        S: AsRef<Path> + zvariant::Type + Serialize + Debug,
    >(
        &self,
        cwd_path: C,
        argv: &[S],
        fds: HashMap<u32, Fd>,
        envs: HashMap<&str, &str>,
        flags: BitFlags<SpawnFlags>,
        options: SpawnOptions,
    ) -> Result<u32, Error> {
        let cwd_path = CString::new(cwd_path.as_ref().as_os_str().as_bytes())
            .expect("The `cwd_path` should not contain a trailing 0 bytes");
        let argv = argv
            .iter()
            .map(|s| {
                CString::new(s.as_ref().as_os_str().as_bytes())
                    .expect("The `argv` should not contain a trailing 0 bytes")
            })
            .collect::<Vec<_>>();
        call_method(
            &self.0,
            "Spawn",
            &(
                cwd_path.as_bytes_with_nul(),
                argv.iter()
                    .map(|c| c.as_bytes_with_nul())
                    .collect::<Vec<_>>(),
                fds,
                envs,
                flags,
                options,
            ),
        )
        .await
    }

    /// This methods let you send a Unix signal to a process that was started
    /// [`spawn()`][`FlatpakProxy::spawn`].
    ///
    /// # Arguments
    ///
    /// * `pid` - The PID of the process to send the signal to.
    /// * `signal` - The signal to send.
    /// * `to_process_group` - Whether to send the signal to the process group.
    ///
    /// # Specifications
    ///
    /// See also [`SpawnSignal`](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html#gdbus-method-org-freedesktop-portal-Flatpak.SpawnSignal).
    #[doc(alias = "SpawnSignal")]
    pub async fn spawn_signal(
        &self,
        pid: u32,
        signal: u32,
        to_process_group: bool,
    ) -> Result<(), Error> {
        call_method(&self.0, "SpawnSignal", &(pid, signal, to_process_group)).await
    }

    /// Flags marking what optional features are available.
    ///
    /// # Specifications
    ///
    /// See also [`supports`](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html#gdbus-property-org-freedesktop-portal-Flatpak.supports).
    pub async fn supports(&self) -> Result<BitFlags<SupportsFlags>, Error> {
        self.inner()
            .get_property::<BitFlags<SupportsFlags>>("supports")
            .await
            .map_err(From::from)
    }
}

/// Monitor if there's an update it and install it.
mod update_monitor;
pub use update_monitor::{UpdateInfo, UpdateMonitorProxy, UpdateProgress, UpdateStatus};

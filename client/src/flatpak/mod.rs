//! # Examples
//!
//! Spawn a process inside of the sandbox, only works in a Flatpak.
//!
//! ```rust,no_run
//! use std::{collections::HashMap, os::fd::BorrowedFd};
//!
//! use ashpd::flatpak::{Flatpak, SpawnFlags, SpawnOptions};
//!
//! async fn run() -> ashpd::Result<()> {
//!     let proxy = Flatpak::new().await?;
//!     let fds: HashMap<_, BorrowedFd<'_>> = HashMap::new();
//!
//!     proxy
//!         .spawn(
//!             "/",
//!             &["contrast"],
//!             fds,
//!             HashMap::new(),
//!             SpawnFlags::CLEAR_ENV | SpawnFlags::NO_NETWORK,
//!             SpawnOptions::default(),
//!         )
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

use std::{
    collections::HashMap,
    fmt::Debug,
    os::fd::{AsFd, OwnedFd},
    path::Path,
};

use futures_util::Stream;
use serde::{Deserialize, Serialize};
use zbus::wire::{
    self, Fd, OwnedObjectPath, Type,
    as_variant::{self, optional},
};

use crate::{Error, FilePath, Pid, flatpak::update_monitor::UpdateMonitor, proxy::Proxy};

/// A bitmask representing the "permissions" of a newly created sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
pub struct SandboxFlags(u32);

bitflags::bitflags! {
    impl SandboxFlags: u32 {
        /// Share the display access (X11, Wayland) with the caller.
        const DISPLAY_ACCESS = 1;
        /// Share the sound access (PulseAudio) with the caller.
        const SOUND_ACCESS = 2;
        /// Share the gpu access with the caller.
        const GPU_ACCESS = 4;
        /// Allow sandbox access to (filtered) session bus.
        const SESSION_BUS_ACCESS = 8;
        /// Allow sandbox access to accessibility bus.
        const ACCESSIBILITY_BUS_ACCESS = 16;
    }
}

#[doc(alias = "XdpSpawnFlags")]
/// Flags affecting the created sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
pub struct SpawnFlags(u32);

bitflags::bitflags! {
    impl SpawnFlags: u32 {
        #[doc(alias = "XDP_SPAWN_FLAG_CLEARENV")]
        /// Clear the environment.
        const CLEAR_ENV = 1;
        #[doc(alias = "XDP_SPAWN_FLAG_LATEST")]
        /// Spawn the latest version of the app.
        const LATEST_VERSION = 2;
        #[doc(alias = "XDP_SPAWN_FLAG_SANDBOX")]
        /// Spawn in a sandbox (equivalent of the sandbox option of `flatpak run`).
        const SANDBOX = 4;
        #[doc(alias = "XDP_SPAWN_FLAG_NO_NETWORK")]
        /// Spawn without network (equivalent of the `unshare=network` option of
        /// `flatpak run`).
        const NO_NETWORK = 8;
        #[doc(alias = "XDP_SPAWN_FLAG_WATCH")]
        /// Kill the sandbox when the caller disappears from the session bus.
        const WATCH_BUS = 16;
        /// Expose the sandbox pids in the callers sandbox, only supported if using
        /// user namespaces for containers (not setuid), see the support property.
        const EXPOSE_PIDS = 32;
        /// Emit a SpawnStarted signal once the sandboxed process has been fully
        /// started.
        const NOTIFY_START = 64;
        /// Expose the sandbox process IDs in the caller's sandbox and the caller's
        /// process IDs in the new sandbox.
        const SHARE_PIDS = 128;
        /// Don't provide app files at `/app` in the new sandbox.
        const EMPTY_APP = 256;
    }
}

/// Flags marking what optional features are available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
pub struct SupportsFlags(u32);

bitflags::bitflags! {
    impl SupportsFlags: u32 {
        /// Supports the expose sandbox pids flag of Spawn.
        const EXPOSE_PIDS = 1;
    }
}

#[derive(Serialize, Type, Debug, Default)]
/// Specified options for a [`Flatpak::spawn`] request.
#[zbus(signature = "dict")]
#[serde(rename_all = "kebab-case")]
pub struct SpawnOptions {
    /// A list of filenames for files inside the sandbox that will be exposed to
    /// the new sandbox, for reading and writing.
    #[serde(with = "as_variant", skip_serializing_if = "Vec::is_empty")]
    sandbox_expose: Vec<String>,
    /// A list of filenames for files inside the sandbox that will be exposed to
    /// the new sandbox, read-only.
    #[serde(with = "as_variant", skip_serializing_if = "Vec::is_empty")]
    sandbox_expose_ro: Vec<String>,
    /// A list of file descriptor for files inside the sandbox that will be
    /// exposed to the new sandbox, for reading and writing.
    #[serde(with = "as_variant", skip_serializing_if = "Vec::is_empty")]
    sandbox_expose_fd: Vec<wire::OwnedFd>,
    /// A list of file descriptor for files inside the sandbox that will be
    /// exposed to the new sandbox, read-only.
    #[serde(with = "as_variant", skip_serializing_if = "Vec::is_empty")]
    sandbox_expose_fd_ro: Vec<wire::OwnedFd>,
    /// Flags affecting the created sandbox.
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    sandbox_flags: Option<SandboxFlags>,
    /// A list of environment variables to remove.
    #[serde(with = "as_variant", skip_serializing_if = "Vec::is_empty")]
    unset_env: Vec<String>,
    /// A file descriptor of the directory that  will be used as `/usr` in the
    /// new sandbox.
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    usr_fd: Option<wire::OwnedFd>,
    /// A file descriptor of the directory that  will be used as `/app` in the
    /// new sandbox.
    #[serde(with = "optional", skip_serializing_if = "Option::is_none")]
    app_fd: Option<wire::OwnedFd>,
}

impl SpawnOptions {
    /// Sets the list of filenames for files to expose the new sandbox.
    /// **Note** absolute paths or subdirectories are not allowed.
    #[must_use]
    pub fn sandbox_expose<P: IntoIterator<Item = I>, I: AsRef<str> + Type + Serialize>(
        mut self,
        sandbox_expose: impl Into<Option<P>>,
    ) -> Self {
        self.sandbox_expose = sandbox_expose
            .into()
            .map(|a| a.into_iter().map(|s| s.as_ref().to_owned()).collect())
            .unwrap_or_default();
        self
    }

    /// Sets the list of filenames for files to expose the new sandbox,
    /// read-only.
    /// **Note** absolute paths or subdirectories are not allowed.
    #[must_use]
    pub fn sandbox_expose_ro<P: IntoIterator<Item = I>, I: AsRef<str> + Type + Serialize>(
        mut self,
        sandbox_expose_ro: impl Into<Option<P>>,
    ) -> Self {
        self.sandbox_expose_ro = sandbox_expose_ro
            .into()
            .map(|a| a.into_iter().map(|s| s.as_ref().to_owned()).collect())
            .unwrap_or_default();
        self
    }

    /// Sets the list of file descriptors of files to expose the new sandbox.
    #[must_use]
    pub fn sandbox_expose_fd<P: IntoIterator<Item = OwnedFd>>(
        mut self,
        sandbox_expose_fd: impl Into<Option<P>>,
    ) -> Self {
        self.sandbox_expose_fd = sandbox_expose_fd
            .into()
            .map(|a| a.into_iter().map(wire::OwnedFd::from).collect())
            .unwrap_or_default();
        self
    }

    /// Sets the list of file descriptors of files to expose the new sandbox,
    /// read-only.
    #[must_use]
    pub fn sandbox_expose_fd_ro<P: IntoIterator<Item = OwnedFd>>(
        mut self,
        sandbox_expose_fd_ro: impl Into<Option<P>>,
    ) -> Self {
        self.sandbox_expose_fd_ro = sandbox_expose_fd_ro
            .into()
            .map(|a| a.into_iter().map(wire::OwnedFd::from).collect())
            .unwrap_or_default();
        self
    }

    /// Sets the created sandbox flags.
    #[must_use]
    pub fn sandbox_flags(mut self, sandbox_flags: impl Into<Option<SandboxFlags>>) -> Self {
        self.sandbox_flags = sandbox_flags.into();
        self
    }

    /// Env variables to unset.
    #[must_use]
    pub fn unset_env<P: IntoIterator<Item = I>, I: AsRef<str> + Type + Serialize>(
        mut self,
        env: impl Into<Option<P>>,
    ) -> Self {
        self.unset_env = env
            .into()
            .map(|a| a.into_iter().map(|s| s.as_ref().to_owned()).collect())
            .unwrap_or_default();
        self
    }

    /// Set a file descriptor of the directory that  will be used as `/usr` in
    /// the new sandbox.
    #[must_use]
    pub fn usr_fd(mut self, fd: impl Into<Option<OwnedFd>>) -> Self {
        self.usr_fd = fd.into().map(|f| f.into());
        self
    }

    /// Set a file descriptor of the directory that  will be used as `/app` in
    /// the new sandbox.
    #[must_use]
    pub fn app_fd(mut self, fd: impl Into<Option<OwnedFd>>) -> Self {
        self.app_fd = fd.into().map(|f| f.into());
        self
    }
}

#[derive(Serialize, Type, Debug, Default)]
/// Specified options for a [`Flatpak::create_update_monitor`] request.
#[zbus(signature = "dict")]
pub struct CreateUpdateMonitorOptions {}

/// The interface exposes some interactions with Flatpak on the host to the
/// sandbox. For example, it allows you to restart the applications or start a
/// more sandboxed instance.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Flatpak`](https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-org.freedesktop.portal.Flatpak).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Flatpak")]
pub struct Flatpak(Proxy<'static>);

impl Flatpak {
    /// Create a new instance of [`Flatpak`].
    pub async fn new() -> Result<Self, Error> {
        let proxy = Proxy::new_flatpak("org.freedesktop.portal.Flatpak").await?;
        Ok(Self(proxy))
    }

    /// Create a new instance of [`Flatpak`].
    pub async fn with_connection(connection: zbus::Connection) -> Result<Self, Error> {
        let proxy =
            Proxy::new_flatpak_with_connection(connection, "org.freedesktop.portal.Flatpak")
                .await?;
        Ok(Self(proxy))
    }

    /// Returns the version of the portal interface.
    pub fn version(&self) -> u32 {
        self.0.version()
    }

    /// Creates an update monitor object that will emit signals
    /// when an update for the caller becomes available, and can be used to
    /// install it.
    ///
    /// # Required version
    ///
    /// The method requires the 2nd version implementation of the portal and
    /// would fail with [`Error::RequiresVersion`] otherwise.
    ///
    /// # Specifications
    ///
    /// See also [`CreateUpdateMonitor`](https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-method-org-freedesktop-portal-Flatpak.CreateUpdateMonitor).
    #[doc(alias = "CreateUpdateMonitor")]
    #[doc(alias = "xdp_portal_update_monitor_start")]
    pub async fn create_update_monitor(
        &self,
        options: CreateUpdateMonitorOptions,
    ) -> Result<UpdateMonitor, Error> {
        let path = self
            .0
            .call_versioned::<OwnedObjectPath>("CreateUpdateMonitor", &(options), 2)
            .await?;

        UpdateMonitor::with_connection(self.connection().clone(), path.into_inner()).await
    }

    /// Emitted when a process starts by [`spawn()`][`Flatpak::spawn`].
    ///
    /// # Specifications
    ///
    /// See also [`SpawnStarted`](https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-signal-org-freedesktop-portal-Flatpak.SpawnStarted).
    #[doc(alias = "SpawnStarted")]
    pub async fn receive_spawn_started(&self) -> Result<impl Stream<Item = (u32, u32)>, Error> {
        self.0.signal("SpawnStarted").await
    }

    /// Emitted when a process started by [`spawn()`][`Flatpak::spawn`]
    /// exits.
    ///
    /// # Specifications
    ///
    /// See also [`SpawnExited`](https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-signal-org-freedesktop-portal-Flatpak.SpawnExited).
    #[doc(alias = "SpawnExited")]
    #[doc(alias = "XdpPortal::spawn-exited")]
    pub async fn receive_spawn_exited(&self) -> Result<impl Stream<Item = (u32, u32)>, Error> {
        self.0.signal("SpawnExited").await
    }

    /// This methods let you start a new instance of your application,
    /// optionally enabling a tighter sandbox.
    ///
    /// # Arguments
    ///
    /// * `cwd_path` - The working directory for the new process.
    /// * `argv` - The argv for the new process, starting with the executable to
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
    /// See also [`Spawn`](https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-method-org-freedesktop-portal-Flatpak.Spawn).
    #[doc(alias = "Spawn")]
    #[doc(alias = "xdp_portal_spawn")]
    pub async fn spawn(
        &self,
        cwd_path: impl AsRef<Path>,
        argv: &[impl AsRef<Path>],
        fds: HashMap<u32, impl AsFd>,
        envs: HashMap<&str, &str>,
        flags: SpawnFlags,
        options: SpawnOptions,
    ) -> Result<u32, Error> {
        let cwd_path = FilePath::new(cwd_path)?;
        let argv = argv
            .iter()
            .map(FilePath::new)
            .collect::<Result<Vec<FilePath>, _>>()?;
        let fds: HashMap<u32, Fd> = fds.iter().map(|(k, val)| (*k, Fd::from(val))).collect();
        self.0
            .call("Spawn", &(cwd_path, argv, fds, envs, flags, options))
            .await
    }

    /// This methods let you send a Unix signal to a process that was started
    /// [`spawn()`][`Flatpak::spawn`].
    ///
    /// # Arguments
    ///
    /// * `pid` - The PID of the process to send the signal to.
    /// * `signal` - The signal to send.
    /// * `to_process_group` - Whether to send the signal to the process group.
    ///
    /// # Specifications
    ///
    /// See also [`SpawnSignal`](https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-method-org-freedesktop-portal-Flatpak.SpawnSignal).
    #[doc(alias = "SpawnSignal")]
    #[doc(alias = "xdp_portal_spawn_signal")]
    pub async fn spawn_signal(
        &self,
        pid: Pid,
        signal: u32,
        to_process_group: bool,
    ) -> Result<(), Error> {
        self.0
            .call("SpawnSignal", &(pid, signal, to_process_group))
            .await
    }

    /// Flags marking what optional features are available.
    ///
    /// # Specifications
    ///
    /// See also [`supports`](https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-property-org-freedesktop-portal-Flatpak.supports).
    #[doc(alias = "supports")]
    pub async fn supported_features(&self) -> Result<SupportsFlags, Error> {
        self.0
            .property_versioned::<SupportsFlags>("supports", 3)
            .await
    }
}

impl std::ops::Deref for Flatpak {
    type Target = zbus::Proxy<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Monitor if there's an update it and install it.
pub mod update_monitor;

/// Provide for a way to execute processes outside of the sandbox
mod development;
pub use development::{Development, HostCommandFlags};

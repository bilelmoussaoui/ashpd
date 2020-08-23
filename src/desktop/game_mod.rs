use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};
#[dbus_proxy(
    interface = "org.freedesktop.portal.GameMode",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications access GameMode from within the sandbox.
///
/// It is analogous to the com.feralinteractive.GameMode interface and will proxy request there,
/// but with additional permission checking and pid mapping.
/// The latter is necessary in the case that sandbox has pid namespace isolation enabled.
/// See the man page for pid_namespaces(7) for more details, but briefly,
/// it means that the sandbox has its own process id namespace which is separated
/// from the one on the host. Thus there will be two separate process ids (pids) within
/// two different namespaces that both identify same process.
/// One id from the pid namespace inside the sandbox and one id from the host pid namespace.
/// Since GameMode expects pids from the host pid namespace but
/// programs inside the sandbox can only know pids from the sandbox namespace,
/// process ids need to be translated from the portal to the host namespace.
/// The portal will do that transparently for all calls where this is necessary.
///
/// Note: GameMode will monitor active clients, i.e. games and other programs that
/// have successfully called 'RegisterGame'. In the event that a client terminates
/// without a call to the 'UnregisterGame' method, GameMode will automatically
/// un-register the client. This might happen with a (small) delay.
trait GameMode {
    /// QueryStatus method
    fn query_status(&self, pid: i32) -> Result<i32>;

    /// QueryStatusByPIDFd method
    fn query_status_by_pidfd(&self, target: RawFd, requester: RawFd) -> Result<i32>;

    /// QueryStatusByPid method
    fn query_status_by_pid(&self, target: i32, requester: i32) -> Result<i32>;

    /// RegisterGame method
    fn register_game(&self, pid: i32) -> Result<i32>;

    /// RegisterGameByPIDFd method
    fn register_game_by_pidfd(&self, target: RawFd, requester: RawFd) -> Result<i32>;

    /// RegisterGameByPid method
    fn register_game_by_pid(&self, target: i32, requester: i32) -> Result<i32>;

    /// UnregisterGame method
    fn unregister_game(&self, pid: i32) -> Result<i32>;

    /// UnregisterGameByPIDFd method
    fn unregister_game_by_pidfd(&self, target: RawFd, requester: RawFd) -> Result<i32>;

    /// UnregisterGameByPid method
    fn unregister_game_by_pid(&self, target: i32, requester: i32) -> Result<i32>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

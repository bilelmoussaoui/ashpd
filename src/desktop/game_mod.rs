use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.GameMode",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait GameMode {
    /// QueryStatus method
    fn query_status(&self, pid: i32) -> Result<i32>;

    /// QueryStatusByPIDFd method
    fn query_status_by_pidfd(
        &self,
        target: std::os::unix::io::RawFd,
        requester: std::os::unix::io::RawFd,
    ) -> Result<i32>;

    /// QueryStatusByPid method
    fn query_status_by_pid(&self, target: i32, requester: i32) -> Result<i32>;

    /// RegisterGame method
    fn register_game(&self, pid: i32) -> Result<i32>;

    /// RegisterGameByPIDFd method
    fn register_game_by_pidfd(
        &self,
        target: std::os::unix::io::RawFd,
        requester: std::os::unix::io::RawFd,
    ) -> Result<i32>;

    /// RegisterGameByPid method
    fn register_game_by_pid(&self, target: i32, requester: i32) -> Result<i32>;

    /// UnregisterGame method
    fn unregister_game(&self, pid: i32) -> Result<i32>;

    /// UnregisterGameByPIDFd method
    fn unregister_game_by_pidfd(
        &self,
        target: std::os::unix::io::RawFd,
        requester: std::os::unix::io::RawFd,
    ) -> Result<i32>;

    /// UnregisterGameByPid method
    fn unregister_game_by_pid(&self, target: i32, requester: i32) -> Result<i32>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

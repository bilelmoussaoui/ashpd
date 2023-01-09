use std::{
    ffi::OsStr,
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
};

#[cfg(feature = "async-std")]
use async_std::{fs::File, prelude::*};
#[cfg(feature = "tokio")]
use tokio::{fs::File, io::AsyncReadExt};

use crate::SESSION;

// Some portals returns paths which are bytes and not a typical string
// as those might be null terminated. This might make sense to provide in form
// of a helper in zvariant
pub(crate) fn path_from_null_terminated(bytes: Vec<u8>) -> PathBuf {
    Path::new(OsStr::from_bytes(bytes.split_last().unwrap().1)).to_path_buf()
}

pub(crate) async fn is_flatpak() -> bool {
    #[cfg(feature = "async-std")]
    {
        async_std::path::PathBuf::from("/.flatpak-info")
            .exists()
            .await
    }
    #[cfg(not(feature = "async-std"))]
    {
        std::path::PathBuf::from("/.flatpak-info").exists()
    }
}

pub(crate) async fn is_snap() -> bool {
    let pid = std::process::id();
    let path = format!("/proc/{pid}/cgroup");
    let mut file = match File::open(path).await {
        Ok(file) => file,
        Err(_) => return false,
    };

    let mut buffer = String::new();
    match file.read_to_string(&mut buffer).await {
        Ok(_) => cgroup_v2_is_snap(&buffer),
        Err(_) => false,
    }
}

fn cgroup_v2_is_snap(cgroups: &str) -> bool {
    cgroups
        .lines()
        .map(|line| {
            let (n, rest) = line.split_once(':')?;
            // Check that n is a number.
            n.parse::<u32>().ok()?;
            let unit = match rest.split_once(':') {
                Some(("", unit)) => Some(unit),
                Some(("freezer", unit)) => Some(unit),
                Some(("name=systemd", unit)) => Some(unit),
                _ => None,
            }?;
            let scope = std::path::Path::new(unit).file_name()?.to_str()?;

            Some(scope.starts_with("snap."))
        })
        .any(|x| x.unwrap_or(false))
}

pub(crate) async fn session_connection() -> zbus::Result<zbus::Connection> {
    if let Some(cnx) = SESSION.get() {
        Ok(cnx.clone())
    } else {
        let cnx = zbus::Connection::session().await?;
        SESSION.set(cnx.clone()).expect("Can't reset a OnceCell");
        Ok(cnx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cgroup_v2_is_snap() {
        let data =
            "0::/user.slice/user-1000.slice/user@1000.service/apps.slice/snap.something.scope\n";
        assert_eq!(cgroup_v2_is_snap(data), true);

        let data = "0::/user.slice/user-1000.slice/user@1000.service/apps.slice\n";
        assert_eq!(cgroup_v2_is_snap(data), false);

        let data = "12:pids:/user.slice/user-1000.slice/user@1000.service
11:perf_event:/
10:net_cls,net_prio:/
9:cpuset:/
8:memory:/user.slice/user-1000.slice/user@1000.service/apps.slice/apps-org.gnome.Terminal.slice/vte-spawn-228ae109-a869-4533-8988-65ea4c10b492.scope
7:rdma:/
6:devices:/user.slice
5:blkio:/user.slice
4:hugetlb:/
3:freezer:/snap.portal-test
2:cpu,cpuacct:/user.slice
1:name=systemd:/user.slice/user-1000.slice/user@1000.service/apps.slice/apps-org.gnome.Terminal.slice/vte-spawn-228ae109-a869-4533-8988-65ea4c10b492.scope
0::/user.slice/user-1000.slice/user@1000.service/apps.slice/apps-org.gnome.Terminal.slice/vte-spawn-228ae109-a869-4533-8988-65ea4c10b492.scope\n";
        assert_eq!(cgroup_v2_is_snap(data), true);
    }
}

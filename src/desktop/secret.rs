use std::os::unix::io::RawFd;
use zbus::{dbus_proxy, fdo::Result};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug)]
/// Specified options on a retrieve secret request.
pub struct ScecretOptions {
    /// A string returned by a pervious call to `retrieve_secret`
    pub token: Option<String>,
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Secret",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications retrieve a per-application secret.
/// The secret can then be used for encrypting confidential data inside the sandbox.
trait Secret {
    /// RetrieveSecret method
    fn retrieve_secret(&self, fd: RawFd, options: ScecretOptions) -> Result<String>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}

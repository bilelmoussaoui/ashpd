use std::collections::HashMap;
use zbus::{dbus_proxy, fdo::Result};

#[dbus_proxy(
    interface = "org.freedesktop.portal.Email",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications request sending an email.
trait Email {
    /// Presents a window that lets the user compose an email.
    ///
    /// Note that the default email client for the host will need to support mailto: URIs following RFC 2368
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `options` - A hashmap
    ///
    ///     * `handle_token` - A string that will be used as the last element of the handle. Must be a valid object path element.
    ///     * `address` - The email address to send to
    ///     * `addresses` - A `[&str]` containing the email adresses to send to
    ///     * `cc` - A `[&str]` containing the email adresses to CC
    ///     * `bcc` -  A `[&str]` containing the email adresses to BCC
    ///     * `subject` - The subject of the email
    ///     * `body` - The body of the email
    ///     * `attachment_fds` - A `[std::os::unix::io::RawFd]` list of file descriptors of file to attach
    ///
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn compose_email(
        &self,
        parent_window: &str,
        options: HashMap<&str, zvariant::Value>,
    ) -> Result<String>;

    /// version property
    #[dbus_proxy(property)]
    fn version(&self) -> Result<u32>;
}

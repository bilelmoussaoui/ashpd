#[cfg(all(feature = "gtk3", feature = "wayland"))]
mod gtk3;
#[cfg(all(feature = "gtk3", feature = "wayland"))]
pub use self::gtk3::Gtk3ActivationToken;

#[cfg(feature = "gtk4_wayland")]
mod gtk4;
#[cfg(feature = "gtk4_wayland")]
pub use self::gtk4::Gtk4ActivationToken;

#[cfg(any(feature = "wayland"))]
mod wayland;
#[cfg(feature = "wayland")]
pub use wayland::WaylandActivationToken;

use serde::{ser::Serializer, Serialize};
use zbus::zvariant::Type;

// TODO
/// See https://wayland.app/protocols/xdg-activation-v1
#[derive(Debug, Type)]
#[zvariant(signature = "s")]
pub enum ActivationToken {
    #[cfg(feature = "wayland")]
    #[doc(hidden)]
    Wayland(WaylandActivationToken),
    #[cfg(feature = "gtk4_wayland")]
    #[doc(hidden)]
    Gtk4(Gtk4ActivationToken),
    #[cfg(all(feature = "gtk3", feature = "wayland"))]
    #[doc(hidden)]
    Gtk3(Gtk3ActivationToken),
    #[doc(hidden)]
    Raw(String),
    #[doc(hidden)]
    None,
}

impl Serialize for ActivationToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl Default for ActivationToken {
    fn default() -> Self {
        Self::None
    }
}

impl ActivationToken {
    #[cfg(feature = "wayland")]
    /// Create an instance of [`ActivationToken`] from a Wayland surface and the
    /// application's id.
    pub async fn from_wayland_surface(
        app_id: &str,
        surface: &wayland_client::protocol::wl_surface::WlSurface,
    ) -> Self {
        if let Some(token) = WaylandActivationToken::from_surface(app_id, surface).await {
            Self::Wayland(token)
        } else {
            Self::default()
        }
    }

    #[cfg(feature = "wayland")]
    /// Create an instance of [`ActivationToken`] from a raw Wayland surface and
    /// the application's id.
    ///
    /// # Safety
    ///
    /// Both pointers have to be valid surface and display pointers. You must
    /// ensure the `display_ptr` lives longer than the returned
    /// `ActivationToken`.
    pub async unsafe fn from_wayland_raw(
        app_id: &str,
        surface_ptr: *mut std::ffi::c_void,
        display_ptr: *mut std::ffi::c_void,
    ) -> Self {
        if let Some(token) =
            WaylandActivationToken::from_raw(app_id, surface_ptr, display_ptr).await
        {
            Self::Wayland(token)
        } else {
            Self::default()
        }
    }

    #[cfg(feature = "gtk4_wayland")]
    // TODO Maybe name from_display.
    /// Creates a [`ActivationToken`] from a [`gtk4::Native`](https://docs.gtk.org/gtk4/class.Native.html).
    pub async fn from_native<N: ::gtk4::glib::IsA<::gtk4::Native>>(
        app_id: &str,
        native: &N,
    ) -> Self {
        if let Some(token) = Gtk4ActivationToken::from_native(app_id, native).await {
            Self::Gtk4(token)
        } else {
            Self::default()
        }
    }

    #[cfg(all(feature = "gtk3", feature = "wayland"))]
    /// Creates a [`ActivationToken`] from a [`IsA<gdk3::Window>`](https://gtk-rs.org/gtk3-rs/stable/latest/docs/gdk/struct.Window.html).
    pub fn from_window(window: &impl ::gtk3::glib::IsA<::gtk3::gdk::Window>) -> Self {
        if let Some(token) = Gtk3ActivationToken::from_window(window) {
            Self::Gtk3(token)
        } else {
            Self::default()
        }
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            #[cfg(feature = "wayland")]
            Self::Wayland(activation_token) => activation_token.token.as_str(),
            #[cfg(feature = "gtk4_wayland")]
            Self::Gtk4(activation_token) => activation_token.wl_token.token.as_str(),
            #[cfg(all(feature = "gtk3", feature = "wayland"))]
            Self::Gtk3(activation_token) => activation_token.token.as_str(),
            Self::Raw(string) => string.as_str(),
            Self::None => "",
        }
    }

    pub(crate) fn into_string(self) -> String {
        match self {
            #[cfg(feature = "wayland")]
            Self::Wayland(activation_token) => activation_token.token,
            #[cfg(feature = "gtk4_wayland")]
            Self::Gtk4(activation_token) => activation_token.wl_token.token,
            #[cfg(all(feature = "gtk3", feature = "wayland"))]
            Self::Gtk3(activation_token) => activation_token.token,
            Self::Raw(string) => string,
            Self::None => "".into(),
        }
    }
}

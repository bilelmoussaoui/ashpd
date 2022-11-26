//! # Examples
//!
//! ```rust,no_run
//! use ashpd::{
//!     desktop::{
//!         dynamic_launcher::{DynamicLauncherProxy, LauncherType, PrepareInstallOptions},
//!         Icon,
//!     },
//!     WindowIdentifier,
//! };
//!
//! async fn run() -> ashpd::Result<()> {
//!     let proxy = DynamicLauncherProxy::new().await?;
//!     let (name, token) = proxy
//!         .prepare_install(
//!             &WindowIdentifier::default(),
//!             "My App",
//!             Icon::from_names(&["dialog-symbolic"]),
//!             PrepareInstallOptions::default(),
//!         )
//!         .await?;
//!
//!     // Name and Icon will be overwritten from what we provided above
//!     // Exec will be overridden to call `flatpak run our-app` if the application is sandboxed
//!     let desktop_entry = r#"
//!         [Desktop Entry]
//!         Comment=My Web App
//!         Type=Application
//!     "#;
//!     proxy
//!         .install(&token, "some_file.desktop", desktop_entry)
//!         .await?;
//!
//!     proxy.uninstall("some_file.desktop").await?;
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;

use enumflags2::{bitflags, BitFlags};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::zvariant::{self, SerializeDict, Type};

use super::{HandleToken, Icon, DESTINATION, PATH};
use crate::{
    helpers::{call_method, call_request_method, session_connection},
    Error, WindowIdentifier,
};

#[bitflags]
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Debug, Copy, Clone, Type)]
#[repr(u32)]
#[doc(alias = "XdpLauncherType")]
pub enum LauncherType {
    #[doc(alias = "XDP_LAUNCHER_APPLICATION")]
    /// A launcher that represents an application
    Application,
    #[doc(alias = "XDP_LAUNCHER_WEBAPP")]
    /// A launcher that represents a web application
    WebApplication,
}

impl Default for LauncherType {
    fn default() -> Self {
        Self::Application
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Type)]
#[zvariant(signature = "s")]
#[serde(rename_all = "lowercase")]
pub enum IconType {
    Png,
    Jpeg,
    Svg,
}

#[derive(Deserialize, Type)]
#[zvariant(signature = "(vsu)")]
pub struct LauncherIcon(Icon, IconType, u32);

impl LauncherIcon {
    pub fn icon(&self) -> &Icon {
        &self.0
    }

    pub fn type_(&self) -> IconType {
        self.1
    }

    pub fn size(&self) -> u32 {
        self.2
    }
}

#[derive(Debug, Default, SerializeDict, Type)]
#[zvariant(signature = "dict")]
pub struct PrepareInstallOptions {
    handle_token: HandleToken,
    modal: Option<bool>,
    launcher_type: LauncherType,
    target: Option<String>,
    editable_name: Option<bool>,
    editable_icon: Option<bool>,
}

impl PrepareInstallOptions {
    pub fn modal(mut self, modal: impl Into<Option<bool>>) -> Self {
        self.modal = modal.into();
        self
    }

    pub fn launcher_type(mut self, launcher_type: LauncherType) -> Self {
        self.launcher_type = launcher_type;
        self
    }

    pub fn target<'a>(mut self, target: impl Into<Option<&'a str>>) -> Self {
        self.target = target.into().map(ToOwned::to_owned);
        self
    }

    pub fn editable_name(mut self, editable_name: impl Into<Option<bool>>) -> Self {
        self.editable_name = editable_name.into();
        self
    }

    pub fn editable_icon(mut self, editable_icon: impl Into<Option<bool>>) -> Self {
        self.editable_icon = editable_icon.into();
        self
    }
}

/// The interface lets sandboxed applications install launchers like Web
/// Application from your browser or Steam.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.DynamicLauncher`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.DynamicLauncher).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.DynamicLauncher")]
pub struct DynamicLauncherProxy<'a>(zbus::Proxy<'a>);

impl<'a> DynamicLauncherProxy<'a> {
    /// Create a new instance of [`DynamicLauncherProxy`].
    pub async fn new() -> Result<DynamicLauncherProxy<'a>, Error> {
        let connection = session_connection().await?;
        let proxy = zbus::ProxyBuilder::new_bare(&connection)
            .interface("org.freedesktop.portal.DynamicLauncher")?
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

    /// # Specifications
    ///
    /// See also [`PrepareInstall`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-DynamicLauncher.PrepareInstall).
    #[doc(alias = "PrepareInstall")]
    #[doc(alias = "xdp_portal_dynamic_launcher_prepare_install")]
    #[doc(alias = "xdp_portal_dynamic_launcher_prepare_install_finish")]
    pub async fn prepare_install(
        &self,
        parent_window: &WindowIdentifier,
        name: &str,
        icon: Icon,
        options: PrepareInstallOptions,
    ) -> Result<(String, String), Error> {
        let response = call_request_method(
            self.inner(),
            &options.handle_token,
            "PrepareInstall",
            &(parent_window, name, icon, &options),
        )
        .await?;
        Ok(response)
    }

    /// # Specifications
    ///
    /// See also [`RequestInstallToken`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-DynamicLauncher.RequestInstallToken).
    #[doc(alias = "RequestInstallToken")]
    #[doc(alias = "xdp_portal_dynamic_launcher_request_install_token")]
    pub async fn request_install_token(&self, name: &str, icon: Icon) -> Result<String, Error> {
        // No supported options for now
        let options: HashMap<&str, zvariant::Value<'_>> = HashMap::new();
        let token =
            call_method::<String, _>(self.inner(), "RequestInstallToken", &(name, icon, options))
                .await?;
        Ok(token)
    }

    /// # Specifications
    ///
    /// See also [`Install`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-DynamicLauncher.Install).
    #[doc(alias = "Install")]
    #[doc(alias = "xdp_portal_dynamic_launcher_install")]
    pub async fn install(
        &self,
        token: &str,
        desktop_file_id: &str,
        desktop_entry: &str,
    ) -> Result<(), Error> {
        // No supported options for now
        let options: HashMap<&str, zvariant::Value<'_>> = HashMap::new();
        call_method(
            self.inner(),
            "Install",
            &(token, desktop_file_id, desktop_entry, options),
        )
        .await?;
        Ok(())
    }

    /// # Specifications
    ///
    /// See also [`Uninstall`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-DynamicLauncher.Uninstall).
    #[doc(alias = "Uninstall")]
    #[doc(alias = "xdp_portal_dynamic_launcher_uninstall")]
    pub async fn uninstall(&self, desktop_file_id: &str) -> Result<(), Error> {
        // No supported options for now
        let options: HashMap<&str, zvariant::Value<'_>> = HashMap::new();
        call_method(self.inner(), "Uninstall", &(desktop_file_id, options)).await?;
        Ok(())
    }

    /// # Specifications
    ///
    /// See also [`GetDesktopEntry`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-DynamicLauncher.GetDesktopEntry).
    #[doc(alias = "GetDesktopEntry")]
    #[doc(alias = "xdp_portal_dynamic_launcher_get_desktop_entry")]
    pub async fn desktop_entry(&self, desktop_file_id: &str) -> Result<String, Error> {
        call_method(self.inner(), "GetDesktopEntry", &(desktop_file_id)).await
    }

    /// # Specifications
    ///
    /// See also [`GetIcon`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-DynamicLauncher.GetIcon).
    #[doc(alias = "GetIcon")]
    #[doc(alias = "xdp_portal_dynamic_launcher_get_icon")]
    pub async fn icon(&self, desktop_file_id: &str) -> Result<LauncherIcon, Error> {
        call_method(self.inner(), "GetIcon", &(desktop_file_id)).await
    }

    /// # Specifications
    ///
    /// See also [`Launch`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-DynamicLauncher.Launch).
    #[doc(alias = "Launch")]
    #[doc(alias = "xdp_portal_dynamic_launcher_launch")]
    pub async fn launch(&self, desktop_file_id: &str) -> Result<(), Error> {
        // TODO: handle activation_token
        let options: HashMap<&str, zvariant::Value<'_>> = HashMap::new();
        call_method(self.inner(), "Launch", &(desktop_file_id, &options)).await
    }

    /// # Specifications
    ///
    /// See also [`SupportedLauncherTypes`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-property-org-freedesktop-portal-DynamicLauncher.SupportedLauncherTypes).
    #[doc(alias = "SupportedLauncherTypes")]
    pub async fn supported_launcher_types(&self) -> Result<BitFlags<LauncherType>, Error> {
        self.inner()
            .get_property::<BitFlags<LauncherType>>("SupportedLauncherTypes")
            .await
            .map_err(From::from)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_icon_signature() {
        let signature = LauncherIcon::signature();
        assert_eq!(signature.as_str(), "(vsu)");

        let icon = vec![IconType::Png];
        assert_eq!(serde_json::to_string(&icon).unwrap(), "[\"png\"]");
    }
}

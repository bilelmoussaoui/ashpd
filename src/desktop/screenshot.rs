//! Take a screenshot or pick a color.
//!
//! Wrapper of the DBus interface: [`org.freedesktop.portal.Screenshot`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-org.freedesktop.portal.Screenshot).
//!
//! # Examples
//!
//! ## Taking a screenshot
//!
//! ```rust,no_run
//! use ashpd::desktop::screenshot::ScreenshotRequest;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let response = ScreenshotRequest::default()
//!         .interactive(true)
//!         .modal(true)
//!         .build()
//!         .await?
//!         .response()?;
//!     println!("URI: {}", response.uri());
//!     Ok(())
//! }
//! ```
//!
//! ## Picking a color
//!
//! ```rust,no_run
//! use ashpd::desktop::screenshot::ColorResponse;
//!
//! async fn run() -> ashpd::Result<()> {
//!     let color = ColorResponse::builder().build().await?.response()?;
//!     println!("({}, {}, {})", color.red(), color.green(), color.blue());
//!
//!     Ok(())
//! }
//! ```
use std::fmt::Debug;

use zbus::zvariant::{DeserializeDict, SerializeDict, Type};

use super::{HandleToken, Request};
use crate::{proxy::Proxy, Error, WindowIdentifier};

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct ScreenshotOptions {
    handle_token: HandleToken,
    modal: Option<bool>,
    interactive: Option<bool>,
}

#[derive(DeserializeDict, Type)]
#[zvariant(signature = "dict")]
pub struct ScreenshotResponse {
    uri: url::Url,
}

impl ScreenshotResponse {
    pub fn uri(&self) -> &url::Url {
        &self.uri
    }
}

impl Debug for ScreenshotResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.uri.as_str())
    }
}

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
struct ColorOptions {
    handle_token: HandleToken,
}

#[derive(DeserializeDict, Clone, Copy, PartialEq, Type)]
/// The response of a [`ColorRequest`] request.
///
/// **Note** the values are normalized.
#[zvariant(signature = "dict")]
pub struct ColorResponse {
    color: [f64; 3],
}

impl ColorResponse {
    /// Red.
    pub fn red(&self) -> f64 {
        self.color[0]
    }

    /// Green.
    pub fn green(&self) -> f64 {
        self.color[1]
    }

    /// Blue.
    pub fn blue(&self) -> f64 {
        self.color[2]
    }

    /// Creates a new builder-pattern struct instance to construct
    /// [`ColorResponse`].
    ///
    /// This method returns an instance of [`ColorRequest`].
    pub fn builder() -> ColorRequest {
        ColorRequest::default()
    }
}

#[cfg(feature = "gtk3")]
impl From<ColorResponse> for gtk3::gdk::RGBA {
    fn from(color: ColorResponse) -> Self {
        gtk3::gdk::RGBA::new(color.red(), color.green(), color.blue(), 1.0)
    }
}

#[cfg(feature = "gtk4")]
impl From<ColorResponse> for gtk4::gdk::RGBA {
    fn from(color: ColorResponse) -> Self {
        gtk4::gdk::RGBA::builder()
            .red(color.red() as f32)
            .green(color.green() as f32)
            .blue(color.blue() as f32)
            .build()
    }
}

impl std::fmt::Debug for ColorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColorResponse")
            .field("red", &self.red())
            .field("green", &self.green())
            .field("blue", &self.blue())
            .finish()
    }
}

impl std::fmt::Display for ColorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "({}, {}, {})",
            self.red(),
            self.green(),
            self.blue()
        ))
    }
}

#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Screenshot")]
struct ScreenshotProxy<'a>(Proxy<'a>);

impl<'a> ScreenshotProxy<'a> {
    /// Create a new instance of [`ScreenshotProxy`].
    pub async fn new() -> Result<ScreenshotProxy<'a>, Error> {
        let proxy = Proxy::new_desktop("org.freedesktop.portal.Screenshot").await?;
        Ok(Self(proxy))
    }

    /// Obtains the color of a single pixel.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    ///
    /// # Specifications
    ///
    /// See also [`PickColor`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Screenshot.PickColor).
    #[doc(alias = "PickColor")]
    #[doc(alias = "xdp_portal_pick_color")]
    pub async fn pick_color(
        &self,
        identifier: &WindowIdentifier,
        options: ColorOptions,
    ) -> Result<Request<'static, ColorResponse>, Error> {
        self.0
            .call_request_method(&options.handle_token, "PickColor", &(&identifier, &options))
            .await
    }

    /// Takes a screenshot.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    /// * `interactive` - Sets whether the dialog should offer customization
    ///   before a screenshot or not.
    /// * `modal` - Sets whether the dialog should be a modal.
    ///
    /// # Returns
    ///
    /// The screenshot URI.
    ///
    /// # Specifications
    ///
    /// See also [`Screenshot`](https://flatpak.github.io/xdg-desktop-portal/index.html#gdbus-method-org-freedesktop-portal-Screenshot.Screenshot).
    #[doc(alias = "Screenshot")]
    #[doc(alias = "xdp_portal_take_screenshot")]
    pub async fn screenshot(
        &self,
        identifier: &WindowIdentifier,
        options: ScreenshotOptions,
    ) -> Result<Request<'static, ScreenshotResponse>, Error> {
        self.0
            .call_request_method(
                &options.handle_token,
                "Screenshot",
                &(&identifier, &options),
            )
            .await
    }
}

#[derive(Debug, Default)]
#[doc(alias = "xdp_portal_pick_color")]
/// A [builder-pattern] type to construct [`ColorResponse`].
///
/// [builder-pattern]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
pub struct ColorRequest {
    identifier: WindowIdentifier,
    options: ColorOptions,
}

impl ColorRequest {
    #[must_use]
    /// Sets a window identifier.
    pub fn identifier(mut self, identifier: WindowIdentifier) -> Self {
        self.identifier = identifier;
        self
    }

    /// Build the [`ColorResponse`].
    pub async fn build(self) -> Result<Request<'static, ColorResponse>, Error> {
        let proxy = ScreenshotProxy::new().await?;
        proxy.pick_color(&self.identifier, self.options).await
    }
}

#[derive(Debug, Default)]
#[doc(alias = "xdp_portal_take_screenshot")]
/// A [builder-pattern] type to construct a screenshot [`Url`].
///
/// [builder-pattern]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
pub struct ScreenshotRequest {
    options: ScreenshotOptions,
    identifier: WindowIdentifier,
}

impl ScreenshotRequest {
    #[must_use]
    /// Sets a window identifier.
    pub fn identifier(mut self, identifier: impl Into<Option<WindowIdentifier>>) -> Self {
        self.identifier = identifier.into().unwrap_or_default();
        self
    }

    /// Sets whether the dialog should be a modal.
    #[must_use]
    pub fn modal(mut self, modal: impl Into<Option<bool>>) -> Self {
        self.options.modal = modal.into();
        self
    }

    /// Sets whether the dialog should offer customization before a screenshot
    /// or not.
    #[must_use]
    pub fn interactive(mut self, interactive: impl Into<Option<bool>>) -> Self {
        self.options.interactive = interactive.into();
        self
    }

    /// Build the [`Url`].
    pub async fn build(self) -> Result<Request<'static, ScreenshotResponse>, Error> {
        let proxy = ScreenshotProxy::new().await?;
        proxy.screenshot(&self.identifier, self.options).await
    }
}

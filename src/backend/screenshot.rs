use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    backend::request::{Request, RequestImpl},
    desktop::{request::Response, screenshot::Screenshot as ScreenshotResponse, Color},
    zvariant::{DeserializeDict, OwnedObjectPath, Type},
    AppID, WindowIdentifierType,
};

#[derive(DeserializeDict, Type, Debug)]
#[zvariant(signature = "dict")]
pub struct ScreenshotOptions {
    modal: Option<bool>,
    interactive: Option<bool>,
    permission_store_checked: Option<bool>,
}

impl ScreenshotOptions {
    pub fn modal(&self) -> Option<bool> {
        self.modal
    }

    pub fn interactive(&self) -> Option<bool> {
        self.interactive
    }

    pub fn permission_store_checked(&self) -> Option<bool> {
        self.permission_store_checked
    }
}

#[derive(DeserializeDict, Type, Debug)]
#[zvariant(signature = "dict")]
pub struct ColorOptions;

#[async_trait]
pub trait ScreenshotImpl: RequestImpl {
    async fn screenshot(
        &self,
        app_id: Option<AppID>,
        window_identifier: Option<WindowIdentifierType>,
        options: ScreenshotOptions,
    ) -> Response<ScreenshotResponse>;

    async fn pick_color(
        &self,
        app_id: Option<AppID>,
        window_identifier: Option<WindowIdentifierType>,
        options: ColorOptions,
    ) -> Response<Color>;
}

pub struct ScreenshotInterface {
    imp: Arc<dyn ScreenshotImpl>,
    cnx: zbus::Connection,
}

impl ScreenshotInterface {
    pub fn new(imp: impl ScreenshotImpl + 'static, cnx: zbus::Connection) -> Self {
        Self {
            imp: Arc::new(imp),
            cnx,
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Screenshot")]
impl ScreenshotInterface {
    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        2
    }

    #[zbus(name = "Screenshot")]
    #[dbus_interface(out_args("response", "results"))]
    async fn screenshot(
        &self,
        handle: OwnedObjectPath,
        app_id: &str,
        window_identifier: &str,
        options: ScreenshotOptions,
    ) -> Response<ScreenshotResponse> {
        let window_identifier = WindowIdentifierType::from_maybe_str(window_identifier);
        let app_id = AppID::from_maybe_str(app_id);
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Screenshot::Screenshot",
            &self.cnx,
            handle,
            Arc::clone(&self.imp),
            async move { imp.screenshot(app_id, window_identifier, options).await },
        )
        .await
        .unwrap_or(Response::other())
    }

    #[zbus(name = "PickColor")]
    #[dbus_interface(out_args("response", "results"))]
    async fn pick_color(
        &self,
        handle: OwnedObjectPath,
        app_id: &str,
        window_identifier: &str,
        options: ColorOptions,
    ) -> Response<Color> {
        let window_identifier = WindowIdentifierType::from_maybe_str(window_identifier);
        let app_id = AppID::from_maybe_str(app_id);
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Screenshot::PickColor",
            &self.cnx,
            handle,
            Arc::clone(&self.imp),
            async move { imp.pick_color(app_id, window_identifier, options).await },
        )
        .await
        .unwrap_or(Response::other())
    }
}

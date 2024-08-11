use std::sync::Arc;

use async_trait::async_trait;
use futures_util::future::abortable;

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
    imp: Arc<Box<dyn ScreenshotImpl>>,
    cnx: zbus::Connection,
}

impl ScreenshotInterface {
    pub fn new(imp: impl ScreenshotImpl + 'static, cnx: zbus::Connection) -> Self {
        Self {
            imp: Arc::new(Box::new(imp)),
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
    async fn screenshot(
        &self,
        #[zbus(object_server)] server: &zbus::object_server::ObjectServer,
        handle: OwnedObjectPath,
        app_id: &str,
        window_identifier: &str,
        options: ScreenshotOptions,
    ) -> Response<ScreenshotResponse> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Screenshot::Screenshot");

        let window_identifier = WindowIdentifierType::from_maybe_str(window_identifier);
        let app_id = AppID::from_maybe_str(app_id);

        let imp: Arc<Box<dyn ScreenshotImpl>> = Arc::clone(&self.imp);
        let (fut, request_handle) =
            abortable(async { imp.screenshot(app_id, window_identifier, options).await });

        let imp_request = Arc::clone(&self.imp);
        let close_cb = || {
            tokio::spawn(async move {
                RequestImpl::close(&**imp_request).await;
            });
        };
        let request = Request::new(close_cb, handle.clone(), request_handle, self.cnx.clone());
        server.at(&handle, request).await.unwrap();

        let response = fut.await.unwrap_or(Response::cancelled());
        #[cfg(feature = "tracing")]
        tracing::debug!("Releasing request {:?}", handle.as_str());
        server.remove::<Request, _>(&handle).await.unwrap();

        #[cfg(feature = "tracing")]
        tracing::debug!("Screenshot::Screenshot returned {:#?}", response);
        response
    }

    #[zbus(name = "PickColor")]
    async fn pick_color(
        &self,
        #[zbus(object_server)] server: &zbus::object_server::ObjectServer,
        handle: OwnedObjectPath,
        app_id: &str,
        window_identifier: &str,
        options: ColorOptions,
    ) -> Response<Color> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Screenshot::PickColor");
        let window_identifier = WindowIdentifierType::from_maybe_str(window_identifier);
        let app_id = AppID::from_maybe_str(app_id);

        let imp: Arc<Box<dyn ScreenshotImpl>> = Arc::clone(&self.imp);
        let (fut, request_handle) =
            abortable(async { imp.pick_color(app_id, window_identifier, options).await });

        let imp_request = Arc::clone(&self.imp);
        let close_cb = || {
            tokio::spawn(async move {
                RequestImpl::close(&**imp_request).await;
            });
        };
        let request = Request::new(close_cb, handle.clone(), request_handle, self.cnx.clone());
        server.at(&handle, request).await.unwrap();

        let response = fut.await.unwrap_or(Response::cancelled());
        #[cfg(feature = "tracing")]
        tracing::debug!("Releasing request {:?}", handle.as_str());
        server.remove::<Request, _>(&handle).await.unwrap();

        #[cfg(feature = "tracing")]
        tracing::debug!("Screenshot::PickColor returned {:#?}", response);
        response
    }
}

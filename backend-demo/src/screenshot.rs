use ashpd::{
    backend::{
        request::RequestImpl,
        screenshot::{ColorOptions, ScreenshotImpl, ScreenshotOptions},
    },
    desktop::{screenshot::Screenshot as ScreenshotResponse, Color, Response},
    AppID, WindowIdentifierType,
};
use async_trait::async_trait;

#[derive(Default)]
pub struct Screenshot;

#[async_trait]
impl RequestImpl for Screenshot {
    async fn close(&self) {
        tracing::debug!("IN Close()");
    }
}

#[async_trait]
impl ScreenshotImpl for Screenshot {
    async fn screenshot(
        &self,
        _app_id: Option<AppID>,
        _window_identifier: Option<WindowIdentifierType>,
        _options: ScreenshotOptions,
    ) -> Response<ScreenshotResponse> {
        Response::ok(ScreenshotResponse::new(
            url::Url::parse("file:///some/sreenshot").unwrap(),
        ))
    }

    async fn pick_color(
        &self,
        _app_id: Option<AppID>,
        _window_identifier: Option<WindowIdentifierType>,
        _options: ColorOptions,
    ) -> Response<Color> {
        Response::ok(Color::new(1.0, 1.0, 1.0))
    }
}

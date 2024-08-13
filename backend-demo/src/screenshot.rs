use ashpd::{
    backend::{
        request::RequestImpl,
        screenshot::{ColorOptions, ScreenshotImpl, ScreenshotOptions},
        Result,
    },
    desktop::{screenshot::Screenshot as ScreenshotResponse, Color},
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
    ) -> Result<ScreenshotResponse> {
        Ok(ScreenshotResponse::new(
            url::Url::parse("file:///some/sreenshot").unwrap(),
        ))
    }

    async fn pick_color(
        &self,
        _app_id: Option<AppID>,
        _window_identifier: Option<WindowIdentifierType>,
        _options: ColorOptions,
    ) -> Result<Color> {
        Ok(Color::new(1.0, 1.0, 1.0))
    }
}

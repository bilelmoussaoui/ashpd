use ashpd::{
    backend::{
        request::RequestImpl,
        screenshot::{ColorOptions, ScreenshotImpl, ScreenshotOptions},
        Result,
    },
    desktop::{screenshot::Screenshot as ScreenshotResponse, Color, HandleToken},
    AppID, WindowIdentifierType,
};
use async_trait::async_trait;

#[derive(Default)]
pub struct Screenshot;

#[async_trait]
impl RequestImpl for Screenshot {
    async fn close(&self, token: HandleToken) {
        tracing::debug!("IN Close(): {token}");
    }
}

#[async_trait]
impl ScreenshotImpl for Screenshot {
    async fn screenshot(
        &self,
        _token: HandleToken,
        _app_id: Option<AppID>,
        _window_identifier: Option<WindowIdentifierType>,
        _options: ScreenshotOptions,
    ) -> Result<ScreenshotResponse> {
        Ok(ScreenshotResponse::new(
            url::Url::parse("file:///some/screenshot").unwrap(),
        ))
    }

    async fn pick_color(
        &self,
        _token: HandleToken,
        _app_id: Option<AppID>,
        _window_identifier: Option<WindowIdentifierType>,
        _options: ColorOptions,
    ) -> Result<Color> {
        Ok(Color::new(1.0, 1.0, 1.0))
    }
}

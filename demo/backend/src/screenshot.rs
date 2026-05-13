use ashpd::{
    MaybeAppID, Uri, WindowIdentifierType,
    backend::{Result, request::RequestImpl, screenshot::ScreenshotImpl},
    desktop::{
        Color, HandleToken,
        screenshot::{
            AvailableTargets, ColorOptions, Screenshot as ScreenshotResponse, ScreenshotOptions,
        },
    },
    enumflags2::BitFlags,
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
    fn available_targets(&self) -> BitFlags<AvailableTargets> {
        AvailableTargets::Window | AvailableTargets::Screen
    }

    async fn screenshot(
        &self,
        _token: HandleToken,
        _app_id: Option<MaybeAppID>,
        _window_identifier: Option<WindowIdentifierType>,
        _options: ScreenshotOptions,
    ) -> Result<ScreenshotResponse> {
        Ok(ScreenshotResponse::new(
            Uri::parse("file:///some/screenshot").unwrap(),
        ))
    }

    async fn pick_color(
        &self,
        _token: HandleToken,
        _app_id: Option<MaybeAppID>,
        _window_identifier: Option<WindowIdentifierType>,
        _options: ColorOptions,
    ) -> Result<Color> {
        Ok(Color::new(1.0, 1.0, 1.0))
    }
}

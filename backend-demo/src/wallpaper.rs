use ashpd::{
    backend::{
        request::RequestImpl,
        wallpaper::{WallpaperImpl, WallpaperOptions},
        Result,
    },
    AppID, WindowIdentifierType,
};
use async_trait::async_trait;

#[derive(Default)]
pub struct Wallpaper;

#[async_trait]
impl RequestImpl for Wallpaper {
    async fn close(&self) {
        tracing::debug!("IN Close()");
    }
}

#[async_trait]
impl WallpaperImpl for Wallpaper {
    async fn with_uri(
        &self,
        _app_id: Option<AppID>,
        _window_identifier: Option<WindowIdentifierType>,
        _uri: url::Url,
        _options: WallpaperOptions,
    ) -> Result<()> {
        Ok(())
    }
}

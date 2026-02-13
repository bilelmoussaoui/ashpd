use ashpd::{
    AppID, Uri, WindowIdentifierType,
    backend::{
        Result,
        request::RequestImpl,
        wallpaper::{WallpaperImpl, WallpaperOptions},
    },
    desktop::HandleToken,
};
use async_trait::async_trait;

#[derive(Default)]
pub struct Wallpaper;

#[async_trait]
impl RequestImpl for Wallpaper {
    async fn close(&self, token: HandleToken) {
        tracing::debug!("IN Close(): {token}");
    }
}

#[async_trait]
impl WallpaperImpl for Wallpaper {
    async fn with_uri(
        &self,
        _token: HandleToken,
        _app_id: Option<AppID>,
        _window_identifier: Option<WindowIdentifierType>,
        _uri: Uri,
        _options: WallpaperOptions,
    ) -> Result<()> {
        Ok(())
    }
}

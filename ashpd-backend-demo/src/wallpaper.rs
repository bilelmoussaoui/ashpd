use ashpd::{
    backend::{
        request::RequestImpl,
        wallpaper::{WallpaperImpl, WallpaperOptions},
    },
    desktop::{wallpaper::SetOn, Response},
    AppID, ExternalWindow, WindowIdentifierType,
};
use async_trait::async_trait;
use gtk::prelude::*;

use crate::wallpaper_preview::WallpaperPreview;

#[derive(Default)]
pub struct Wallpaper;

const BACKGROUND_SCHEMA: &str = "org.gnome.desktop.background";

#[async_trait]
impl RequestImpl for Wallpaper {
    async fn close(&self) {
        log::debug!("IN Close()");
    }
}

#[async_trait]
impl WallpaperImpl for Wallpaper {
    async fn set_wallpaper_uri(
        &self,
        app_id: AppID,
        window_identifier: WindowIdentifierType,
        uri: url::Url,
        options: WallpaperOptions,
    ) -> Response<()> {
        log::debug!(
            "IN SetWallpaperURI({app_id}, {window_identifier:?}, {}, {options:?})",
            uri.as_str()
        );
        let flow = if options.show_preview() {
            log::debug!("Opening wallpaper preview");
            let preview = WallpaperPreview::default();
            preview.set_uri(&uri);
            {
                let external_window = ExternalWindow::new(window_identifier);
                let fake_window = ExternalWindow::fake(external_window.as_ref());
                preview.set_transient_for(Some(&fake_window));

                if let Some(ref external) = external_window {
                    gtk::Widget::realize(preview.upcast_ref());
                    external.set_parent_of(&preview.surface().unwrap());
                }
            }
            preview.present_and_wait().await
        } else {
            std::ops::ControlFlow::Continue(())
        };

        let response = if flow.is_break() {
            Response::cancelled()
        } else if set_gsetting(uri, options.set_on()).is_ok() {
            Response::ok(())
        } else {
            Response::other()
        };

        log::debug!("OUT SetWallpaperURI({response:?})",);
        response
    }
}

fn set_gsetting(uri: url::Url, _set_on: SetOn) -> anyhow::Result<()> {
    let settings = gtk::gio::Settings::new(BACKGROUND_SCHEMA);
    // TODO: handle set_on if possible
    settings.set_string("picture-uri", uri.as_str())?;
    settings.set_string("picture-uri-dark", uri.as_str())?;

    Ok(())
}

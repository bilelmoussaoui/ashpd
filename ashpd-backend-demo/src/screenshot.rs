use ashpd::{
    backend::{
        request::RequestImpl,
        screenshot::{ColorOptions, ScreenshotImpl, ScreenshotOptions},
    },
    desktop::{screenshot::Screenshot as ScreenshotResponse, Color, Response},
    AppID, WindowIdentifierType,
};
use async_trait::async_trait;

mod shell_screenshot {

    use super::Color;

    #[zbus::proxy(
        interface = "org.gnome.Shell.Screenshot",
        default_service = "org.gnome.Shell.Screenshot",
        default_path = "/org/gnome/Shell/Screenshot",
        gen_blocking = false
    )]
    pub trait Screenshot {
        #[zbus(name = "PickColor")]
        async fn pick_color(&self) -> zbus::Result<Color>;

        #[zbus(name = "SelectArea")]
        async fn select_area(&self) -> zbus::Result<(i32, i32, i32, i32)>;

        #[zbus(name = "Screenshot")]
        async fn screenshot(
            &self,
            include_cursor: bool,
            flash: bool,
            filename: &str,
        ) -> zbus::Result<(bool, url::Url)>;

        #[zbus(name = "ScreenshotWindow")]
        async fn screenshot_window(
            &self,
            include_frame: bool,
            include_cursor: bool,
            flash: bool,
            filename: &str,
        ) -> zbus::Result<(bool, url::Url)>;

        #[zbus(name = "ScreenshotArea")]
        async fn screenshot_area(
            &self,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            flash: bool,
            filename: &str,
        ) -> zbus::Result<(bool, url::Url)>;
    }

    impl<'a> ScreenshotProxy<'a> {
        pub async fn pick() -> zbus::Result<Color> {
            let cnx = zbus::Connection::session().await?;
            let proxy = ScreenshotProxy::new(&cnx).await?;
            proxy.pick_color().await
        }

        pub async fn screenshot_all() -> zbus::Result<url::Url> {
            let cnx = zbus::Connection::session().await?;
            let proxy = ScreenshotProxy::new(&cnx).await?;
            let (success, uri) = proxy.screenshot(true, true, "").await?;
            Ok(uri)
        }
    }
}

#[derive(Default)]
pub struct Screenshot;

#[async_trait]
impl RequestImpl for Screenshot {
    async fn close(&self) {
        log::debug!("IN Close()");
    }
}

#[async_trait]
impl ScreenshotImpl for Screenshot {
    async fn screenshot(
        &self,
        _app_id: AppID,
        _window_identifier: WindowIdentifierType,
        _options: ScreenshotOptions,
    ) -> Response<ScreenshotResponse> {
        log::debug!("Taking a screenshot");
        // TODO: add a desktop file for this to work
        match shell_screenshot::ScreenshotProxy::screenshot_all().await {
            Ok(uri) => {
                log::debug!("screenshot taken {uri}");
                Response::ok(ScreenshotResponse::new(uri))
            }
            Err(err) => {
                log::error!("Failed to take a screenshot {err}");
                Response::other()
            }
        }
    }

    async fn pick_color(
        &self,
        _app_id: AppID,
        _window_identifier: WindowIdentifierType,
        _options: ColorOptions,
    ) -> Response<Color> {
        log::debug!("Picking color");
        match shell_screenshot::ScreenshotProxy::pick().await {
            Ok(color) => {
                log::debug!("Color picked {color}");
                Response::ok(color)
            }
            Err(err) => {
                log::error!("Failed to pick color {err}");
                Response::other()
            }
        }
    }
}

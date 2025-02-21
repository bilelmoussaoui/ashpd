use ashpd::{
    backend::{
        request::RequestImpl,
        screenshot::{ColorOptions, ScreenshotImpl, ScreenshotOptions},
        Result,
    },
    desktop::{screenshot::Screenshot as ScreenshotResponse, Color, HandleToken},
    AppID, PortalError, WindowIdentifierType,
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
        let path = xdg_user::pictures().unwrap_or(None);
        let mut path = match path {
            Some(p) => std::path::PathBuf::from(p),
            None => {
                return Err(PortalError::Failed(format!(
                    "No XDG pictures directory to save screenshot to"
                )))
            }
        };
        path.push(format!(
            "Screenshot_{}.png",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        ));
        let url = match url::Url::from_file_path(path.as_path()) {
            Ok(url) => url,
            _ => {
                return Err(PortalError::Failed(format!(
                    "Invalid file path: {}",
                    path.display()
                )))
            }
        };
        // The following block is commented out because under Wayland xcap
        // calls into the org.freedesktop.portal.Screenshot D-Bus API. In a
        // scenario where this backend demo portal is installed and handles
        // screenshot requests, it would end up invoking itself recursively.
        // let monitors = xcap::Monitor::all().unwrap_or(vec![]);
        // if monitors.is_empty() {
        // return Err(PortalError::Failed(format!("No monitors found")));
        // }
        // let capture = monitors.first().unwrap().capture_image();
        // let _ = match capture {
        // Ok(image) => image.save_with_format(path, image::ImageFormat::Png),
        // _ => return Err(PortalError::Failed(format!("Failure to take screenshot"))),
        // };
        Ok(ScreenshotResponse::new(url))
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

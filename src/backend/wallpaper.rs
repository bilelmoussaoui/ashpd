use std::sync::Arc;

use async_trait::async_trait;
use futures_util::future::abortable;

use crate::{
    backend::request::{Request, RequestImpl},
    desktop::{
        request::{Response, ResponseType},
        wallpaper::SetOn,
    },
    zvariant::{DeserializeDict, OwnedObjectPath, Type},
    AppID, WindowIdentifierType,
};

#[derive(DeserializeDict, Type, Debug)]
#[zvariant(signature = "dict")]
pub struct WallpaperOptions {
    #[zvariant(rename = "show-preview")]
    show_preview: Option<bool>,
    #[zvariant(rename = "set-on")]
    set_on: Option<SetOn>,
}

impl WallpaperOptions {
    pub fn show_preview(&self) -> Option<bool> {
        self.show_preview
    }

    pub fn set_on(&self) -> Option<SetOn> {
        self.set_on
    }
}

#[async_trait]
pub trait WallpaperImpl: RequestImpl {
    async fn with_uri(
        &self,
        app_id: Option<AppID>,
        window_identifier: Option<WindowIdentifierType>,
        uri: url::Url,
        options: WallpaperOptions,
    ) -> Response<()>;
}

pub struct WallpaperInterface {
    imp: Arc<Box<dyn WallpaperImpl>>,
    cnx: zbus::Connection,
}

impl WallpaperInterface {
    pub fn new(imp: impl WallpaperImpl + 'static, cnx: zbus::Connection) -> Self {
        Self {
            imp: Arc::new(Box::new(imp)),
            cnx,
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Wallpaper")]
impl WallpaperInterface {
    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        1
    }

    #[zbus(name = "SetWallpaperURI")]
    async fn set_wallpaper_uri(
        &self,
        #[zbus(object_server)] server: &zbus::object_server::ObjectServer,
        handle: OwnedObjectPath,
        app_id: &str,
        window_identifier: &str,
        uri: url::Url,
        options: WallpaperOptions,
    ) -> ResponseType {
        #[cfg(feature = "tracing")]
        tracing::debug!("Wallpaper::SetWallpaperURI");
        let window_identifier = WindowIdentifierType::from_maybe_str(window_identifier);
        let app_id = AppID::from_maybe_str(app_id);

        let imp: Arc<Box<dyn WallpaperImpl>> = Arc::clone(&self.imp);
        let (fut, request_handle) =
            abortable(async { imp.with_uri(app_id, window_identifier, uri, options).await });

        let imp_request = Arc::clone(&self.imp);
        let close_cb = || {
            tokio::spawn(async move {
                RequestImpl::close(&**imp_request).await;
            });
        };
        let request = Request::new(close_cb, handle.clone(), request_handle, self.cnx.clone());
        server.at(&handle, request).await.unwrap();

        let response = fut.await.unwrap_or(Response::cancelled()).response_type();
        #[cfg(feature = "tracing")]
        tracing::debug!("Releasing request {:?}", handle.as_str());
        server.remove::<Request, _>(&handle).await.unwrap();

        #[cfg(feature = "tracing")]
        tracing::debug!("Wallpaper::SetWallpaperURI returned {:#?}", response);
        response
    }
}

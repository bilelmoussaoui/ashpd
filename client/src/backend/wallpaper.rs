use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;

use crate::{
    AppID, Uri, WindowIdentifierType,
    backend::{
        Result,
        request::{Request, RequestImpl},
    },
    desktop::{HandleToken, request::ResponseType, wallpaper::SetOn},
    zvariant::{Optional, OwnedObjectPath, Type, as_value::optional},
};

#[derive(Deserialize, Type, Debug)]
#[zvariant(signature = "dict")]
#[serde(rename_all = "kebab-case")]
pub struct WallpaperOptions {
    #[serde(default, with = "optional")]
    show_preview: Option<bool>,
    #[serde(default, with = "optional")]
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
    #[doc(alias = "SetWallpaperURI")]
    async fn with_uri(
        &self,
        token: HandleToken,
        app_id: Option<AppID>,
        window_identifier: Option<WindowIdentifierType>,
        uri: Uri,
        options: WallpaperOptions,
    ) -> Result<()>;
}

pub(crate) struct WallpaperInterface {
    imp: Arc<dyn WallpaperImpl>,
    spawn: Arc<dyn futures_util::task::Spawn + Send + Sync>,
    cnx: zbus::Connection,
}

impl WallpaperInterface {
    pub fn new(
        imp: Arc<dyn WallpaperImpl>,
        cnx: zbus::Connection,
        spawn: Arc<dyn futures_util::task::Spawn + Send + Sync>,
    ) -> Self {
        Self { imp, cnx, spawn }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Wallpaper")]
impl WallpaperInterface {
    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        1
    }

    #[zbus(name = "SetWallpaperURI")]
    #[zbus(out_args("response"))]
    async fn set_wallpaper_uri(
        &self,
        handle: OwnedObjectPath,
        app_id: Optional<AppID>,
        window_identifier: Optional<WindowIdentifierType>,
        uri: Uri,
        options: WallpaperOptions,
    ) -> Result<ResponseType> {
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Wallpaper::SetWallpaperURI",
            &self.cnx,
            handle.clone(),
            Arc::clone(&self.imp),
            Arc::clone(&self.spawn),
            async move {
                imp.with_uri(
                    HandleToken::try_from(&handle).unwrap(),
                    app_id.into(),
                    window_identifier.into(),
                    uri,
                    options,
                )
                .await
            },
        )
        .await
        .map(|r| r.response_type())
    }
}

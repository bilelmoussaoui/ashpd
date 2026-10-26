use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    MaybeAppID, Uri, WindowIdentifierType,
    backend::{
        IfaceInner, Result,
        request::{Request, RequestImpl},
    },
    desktop::{HandleToken, request::ResponseType, wallpaper::WallpaperOptions},
    wire::{Optional, OwnedObjectPath},
};

#[async_trait]
pub trait WallpaperImpl: RequestImpl {
    #[doc(alias = "SetWallpaperURI")]
    async fn with_uri(
        &self,
        token: HandleToken,
        app_id: Option<MaybeAppID>,
        window_identifier: Option<WindowIdentifierType>,
        uri: Uri,
        options: WallpaperOptions,
    ) -> Result<()>;
}

pub(crate) struct WallpaperInterface {
    imp: Arc<dyn WallpaperImpl>,
    inner: IfaceInner,
}

impl WallpaperInterface {
    pub fn new(imp: Arc<dyn WallpaperImpl>, inner: IfaceInner) -> Self {
        Self { imp, inner }
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
        app_id: Optional<MaybeAppID>,
        window_identifier: Optional<WindowIdentifierType>,
        uri: Uri,
        options: WallpaperOptions,
    ) -> Result<ResponseType> {
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Wallpaper::SetWallpaperURI",
            &self.inner,
            handle.clone(),
            Arc::clone(&self.imp),
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

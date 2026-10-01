use std::sync::Arc;

use async_trait::async_trait;
use enumflags2::BitFlags;
use zbus::message::Header;

use crate::{
    MaybeAppID, WindowIdentifierType,
    backend::{
        Result,
        caller::CallerAuthorization,
        request::{Request, RequestImpl},
    },
    desktop::{
        Color, HandleToken,
        request::Response,
        screenshot::{
            AvailableTargets, ColorOptions, Screenshot as ScreenshotResponse, ScreenshotOptions,
        },
    },
    zvariant::{Optional, OwnedObjectPath},
};

#[async_trait]
pub trait ScreenshotImpl: RequestImpl {
    #[doc(alias = "AvailableTargets")]
    fn available_targets(&self) -> BitFlags<AvailableTargets>;

    #[doc(alias = "Screenshot")]
    async fn screenshot(
        &self,
        token: HandleToken,
        app_id: Option<MaybeAppID>,
        window_identifier: Option<WindowIdentifierType>,
        options: ScreenshotOptions,
    ) -> Result<ScreenshotResponse>;

    #[doc(alias = "PickColor")]
    async fn pick_color(
        &self,
        token: HandleToken,
        app_id: Option<MaybeAppID>,
        window_identifier: Option<WindowIdentifierType>,
        options: ColorOptions,
    ) -> Result<Color>;
}

pub(crate) struct ScreenshotInterface {
    imp: Arc<dyn ScreenshotImpl>,
    spawn: Arc<dyn futures_util::task::Spawn + Send + Sync>,
    cnx: zbus::Connection,
    caller_authorization: Arc<CallerAuthorization>,
}

impl ScreenshotInterface {
    pub fn new(
        imp: Arc<dyn ScreenshotImpl>,
        cnx: zbus::Connection,
        spawn: Arc<dyn futures_util::task::Spawn + Send + Sync>,
        caller_authorization: Arc<CallerAuthorization>,
    ) -> Self {
        Self {
            imp,
            cnx,
            spawn,
            caller_authorization,
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Screenshot")]
impl ScreenshotInterface {
    #[zbus(property(emits_changed_signal = "const"), name = "AvailableTargets")]
    fn available_targets(&self) -> u32 {
        self.imp.available_targets().bits()
    }

    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        3
    }

    #[zbus(name = "Screenshot")]
    #[zbus(out_args("response", "results"))]
    async fn screenshot(
        &self,
        handle: OwnedObjectPath,
        app_id: Optional<MaybeAppID>,
        window_identifier: Optional<WindowIdentifierType>,
        options: ScreenshotOptions,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<Response<ScreenshotResponse>> {
        self.caller_authorization
            .authorize(&self.cnx, &header)
            .await?;
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Screenshot::Screenshot",
            &self.cnx,
            Arc::clone(&self.caller_authorization),
            handle.clone(),
            Arc::clone(&self.imp),
            Arc::clone(&self.spawn),
            async move {
                imp.screenshot(
                    HandleToken::try_from(&handle).unwrap(),
                    app_id.into(),
                    window_identifier.into(),
                    options,
                )
                .await
            },
        )
        .await
    }

    #[zbus(name = "PickColor")]
    #[zbus(out_args("response", "results"))]
    async fn pick_color(
        &self,
        handle: OwnedObjectPath,
        app_id: Optional<MaybeAppID>,
        window_identifier: Optional<WindowIdentifierType>,
        options: ColorOptions,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<Response<Color>> {
        self.caller_authorization
            .authorize(&self.cnx, &header)
            .await?;
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Screenshot::PickColor",
            &self.cnx,
            Arc::clone(&self.caller_authorization),
            handle.clone(),
            Arc::clone(&self.imp),
            Arc::clone(&self.spawn),
            async move {
                imp.pick_color(
                    HandleToken::try_from(&handle).unwrap(),
                    app_id.into(),
                    window_identifier.into(),
                    options,
                )
                .await
            },
        )
        .await
    }
}

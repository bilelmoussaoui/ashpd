use std::sync::Arc;

use async_trait::async_trait;
use futures_channel::{
    mpsc::{UnboundedReceiver as Receiver, UnboundedSender as Sender},
    oneshot,
};
use futures_util::{
    future::{try_select, Either},
    pin_mut, SinkExt, StreamExt,
};
use tokio::sync::Mutex;

use crate::{
    backend::{
        request::{Request, RequestImpl},
        Backend,
    },
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
pub trait WallpaperImpl {
    async fn with_uri(
        &self,
        app_id: AppID,
        window_identifier: Option<WindowIdentifierType>,
        uri: url::Url,
        options: WallpaperOptions,
    ) -> Response<()>;
}

pub struct Wallpaper<T: WallpaperImpl + RequestImpl> {
    receiver: Arc<Mutex<Receiver<Action>>>,
    imp: Arc<T>,
    cnx: zbus::Connection,
}

impl<T: WallpaperImpl + RequestImpl> Wallpaper<T> {
    pub async fn new(imp: T, backend: &Backend) -> zbus::Result<Self> {
        let (sender, receiver) = futures_channel::mpsc::unbounded();
        let iface = WallpaperInterface::new(sender);
        backend.serve(iface).await?;
        let provider = Self {
            receiver: Arc::new(Mutex::new(receiver)),
            imp: Arc::new(imp),
            cnx: backend.cnx().clone(),
        };

        Ok(provider)
    }

    async fn activate(&self, action: Action) -> Result<(), crate::Error> {
        let Action::SetWallpaperURI(handle_path, app_id, window_identifier, uri, options, sender) =
            action;
        let request = Request::new(Arc::clone(&self.imp), handle_path, &self.cnx).await?;
        let imp = Arc::clone(&self.imp);
        let future1 = async {
            let result = imp.with_uri(app_id, window_identifier, uri, options).await;
            let _ = sender.send(result);
            Ok(()) as Result<(), crate::Error>
        };
        let future2 = async {
            request.next().await?;
            Ok(()) as Result<(), crate::Error>
        };

        pin_mut!(future1); // 'select' requires Future + Unpin bounds
        pin_mut!(future2);
        match try_select(future1, future2).await {
            Ok(_) => Ok(()),
            Err(Either::Left((err, _))) => Err(err),
            Err(Either::Right((err, _))) => Err(err),
        }?;
        Ok(())
    }

    pub async fn try_next(&self) -> Result<(), crate::Error> {
        if let Some(action) = (*self.receiver.lock().await).next().await {
            self.activate(action).await?;
        }
        Ok(())
    }
}

enum Action {
    SetWallpaperURI(
        OwnedObjectPath,
        AppID,
        Option<WindowIdentifierType>,
        url::Url,
        WallpaperOptions,
        oneshot::Sender<Response<()>>,
    ),
}

struct WallpaperInterface {
    sender: Arc<Mutex<Sender<Action>>>,
}

impl WallpaperInterface {
    pub fn new(sender: Sender<Action>) -> Self {
        Self {
            sender: Arc::new(Mutex::new(sender)),
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Wallpaper")]
impl WallpaperInterface {
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        1
    }

    #[zbus(name = "SetWallpaperURI")]
    async fn set_wallpaper_uri(
        &self,
        handle: OwnedObjectPath,
        app_id: AppID,
        window_identifier: &str,
        uri: url::Url,
        options: WallpaperOptions,
    ) -> ResponseType {
        #[cfg(feature = "tracing")]
        tracing::debug!("Wallpaper::SetWallpaperURI");

        let (sender, receiver) = futures_channel::oneshot::channel();
        let window_identifier = if window_identifier.is_empty() {
            None
        } else {
            window_identifier.parse::<WindowIdentifierType>().ok()
        };

        let _ = self
            .sender
            .lock()
            .await
            .send(Action::SetWallpaperURI(
                handle,
                app_id,
                window_identifier,
                uri,
                options,
                sender,
            ))
            .await;
        let response = receiver
            .await
            .unwrap_or(Response::cancelled())
            .response_type();

        #[cfg(feature = "tracing")]
        tracing::debug!("Wallpaper::SetWallpaperURI returned {:#?}", response);
        response
    }
}

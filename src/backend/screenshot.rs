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
    desktop::{request::Response, screenshot::Screenshot as ScreenshotResponse, Color},
    zvariant::{DeserializeDict, OwnedObjectPath, Type},
    AppID, WindowIdentifierType,
};

#[derive(DeserializeDict, Type, Debug)]
#[zvariant(signature = "dict")]
pub struct ScreenshotOptions {
    modal: Option<bool>,
    interactive: Option<bool>,
    permission_store_checked: Option<bool>,
}

impl ScreenshotOptions {
    pub fn modal(&self) -> Option<bool> {
        self.modal
    }

    pub fn interactive(&self) -> Option<bool> {
        self.interactive
    }

    pub fn permission_store_checked(&self) -> Option<bool> {
        self.permission_store_checked
    }
}

#[derive(DeserializeDict, Type, Debug)]
#[zvariant(signature = "dict")]
pub struct ColorOptions;

#[async_trait]
pub trait ScreenshotImpl {
    async fn screenshot(
        &self,
        app_id: Option<AppID>,
        window_identifier: Option<WindowIdentifierType>,
        options: ScreenshotOptions,
    ) -> Response<ScreenshotResponse>;

    async fn pick_color(
        &self,
        app_id: Option<AppID>,
        window_identifier: Option<WindowIdentifierType>,
        options: ColorOptions,
    ) -> Response<Color>;
}

pub struct Screenshot<T: ScreenshotImpl + RequestImpl> {
    receiver: Arc<Mutex<Receiver<Action>>>,
    imp: Arc<T>,
    cnx: zbus::Connection,
}

impl<T: ScreenshotImpl + RequestImpl> Screenshot<T> {
    pub async fn new(imp: T, backend: &Backend) -> zbus::Result<Self> {
        let (sender, receiver) = futures_channel::mpsc::unbounded();
        let iface = ScreenshotInterface::new(sender);
        backend.serve(iface).await?;
        let provider = Self {
            receiver: Arc::new(Mutex::new(receiver)),
            imp: Arc::new(imp),
            cnx: backend.cnx().clone(),
        };

        Ok(provider)
    }

    async fn activate(&self, action: Action) -> Result<(), crate::Error> {
        match action {
            Action::Screenshot(handle_path, app_id, window_identifier, options, sender) => {
                let future1 = async {
                    let result = self
                        .imp
                        .screenshot(app_id, window_identifier, options)
                        .await;
                    let _ = sender.send(result);
                    Ok(()) as Result<(), crate::Error>
                };

                let request = Request::new(Arc::clone(&self.imp), handle_path, &self.cnx).await?;

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
            }
            Action::PickColor(handle_path, app_id, window_identifier, options, sender) => {
                let future1 = async {
                    let result = self
                        .imp
                        .pick_color(app_id, window_identifier, options)
                        .await;
                    let _ = sender.send(result);
                    Ok(()) as Result<(), crate::Error>
                };

                let request = Request::new(Arc::clone(&self.imp), handle_path, &self.cnx).await?;

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
            }
        };
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
    Screenshot(
        OwnedObjectPath,
        Option<AppID>,
        Option<WindowIdentifierType>,
        ScreenshotOptions,
        oneshot::Sender<Response<ScreenshotResponse>>,
    ),
    PickColor(
        OwnedObjectPath,
        Option<AppID>,
        Option<WindowIdentifierType>,
        ColorOptions,
        oneshot::Sender<Response<Color>>,
    ),
}

struct ScreenshotInterface {
    sender: Arc<Mutex<Sender<Action>>>,
}

impl ScreenshotInterface {
    pub fn new(sender: Sender<Action>) -> Self {
        Self {
            sender: Arc::new(Mutex::new(sender)),
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Screenshot")]
impl ScreenshotInterface {
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        2
    }

    #[zbus(name = "Screenshot")]
    async fn screenshot(
        &self,
        handle: OwnedObjectPath,
        app_id: &str,
        window_identifier: &str,
        options: ScreenshotOptions,
    ) -> Response<ScreenshotResponse> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Screenshot::Screenshot");
        let (sender, receiver) = futures_channel::oneshot::channel();

        let window_identifier = WindowIdentifierType::from_maybe_str(window_identifier);
        let app_id = AppID::from_maybe_str(app_id);

        let _ = self
            .sender
            .lock()
            .await
            .send(Action::Screenshot(
                handle,
                app_id,
                window_identifier,
                options,
                sender,
            ))
            .await;
        let response = receiver.await.unwrap_or(Response::cancelled());
        #[cfg(feature = "tracing")]
        tracing::debug!("Screenshot::Screenshot returned {:#?}", response);
        response
    }

    #[zbus(name = "PickColor")]
    async fn pick_color(
        &self,
        handle: OwnedObjectPath,
        app_id: &str,
        window_identifier: &str,
        options: ColorOptions,
    ) -> Response<Color> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Screenshot::PickColor");

        let (sender, receiver) = futures_channel::oneshot::channel();

        let window_identifier = WindowIdentifierType::from_maybe_str(window_identifier);
        let app_id = AppID::from_maybe_str(app_id);

        let _ = self
            .sender
            .lock()
            .await
            .send(Action::PickColor(
                handle,
                app_id,
                window_identifier,
                options,
                sender,
            ))
            .await;
        let response = receiver.await.unwrap_or(Response::cancelled());
        #[cfg(feature = "tracing")]
        tracing::debug!("Screenshot::PickColor returned {:#?}", response);
        response
    }
}

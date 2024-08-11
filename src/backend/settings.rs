use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use futures_channel::{
    mpsc::{Receiver, Sender},
    oneshot,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::Mutex;

use crate::{
    backend::Backend,
    desktop::{
        settings::{
            ColorScheme, Contrast, Namespace, ACCENT_COLOR_SCHEME_KEY, APPEARANCE_NAMESPACE,
            COLOR_SCHEME_KEY, CONTRAST_KEY,
        },
        Color,
    },
    zbus::SignalContext,
    zvariant::{OwnedValue, Value},
    PortalError,
};

#[async_trait]
pub trait SettingsImpl {
    async fn read_all(&self, namespaces: Vec<String>) -> HashMap<String, Namespace>;

    async fn read(&self, namespace: &str, key: &str) -> Result<OwnedValue, PortalError>;
}

#[derive(Clone)]
pub struct Settings<T: SettingsImpl> {
    receiver: Arc<Mutex<Receiver<Action>>>,
    imp: T,
    cnx: zbus::Connection,
}

impl<T: SettingsImpl> Settings<T> {
    pub async fn new(imp: T, backend: &Backend) -> zbus::Result<Self> {
        let (sender, receiver) = futures_channel::mpsc::channel(10);
        let iface = SettingsInterface::new(sender);
        backend.serve(iface).await?;
        let provider = Self {
            receiver: Arc::new(Mutex::new(receiver)),
            imp,
            cnx: backend.cnx().clone(),
        };

        Ok(provider)
    }

    async fn activate(&self, action: Action) -> Result<(), crate::Error> {
        match action {
            Action::ReadAll(namespaces, sender) => {
                let results = self.imp.read_all(namespaces).await;
                let _ = sender.send(results);
            }
            Action::Read(namespace, key, sender) => {
                let results = self.imp.read(&namespace, &key).await;
                let _ = sender.send(results);
            }
        }

        Ok(())
    }

    pub async fn contrast_changed(&self, contrast: Contrast) -> zbus::Result<()> {
        self.changed(
            APPEARANCE_NAMESPACE,
            CONTRAST_KEY,
            OwnedValue::from(contrast).into(),
        )
        .await
    }

    pub async fn accent_color_changed(&self, color: Color) -> zbus::Result<()> {
        self.changed(
            APPEARANCE_NAMESPACE,
            ACCENT_COLOR_SCHEME_KEY,
            OwnedValue::try_from(color).unwrap().into(),
        )
        .await
    }

    pub async fn color_scheme_changed(&self, scheme: ColorScheme) -> zbus::Result<()> {
        self.changed(
            APPEARANCE_NAMESPACE,
            COLOR_SCHEME_KEY,
            OwnedValue::from(scheme).into(),
        )
        .await
    }

    pub async fn changed(&self, namespace: &str, key: &str, value: Value<'_>) -> zbus::Result<()> {
        let object_server = self.cnx.object_server();
        let iface_ref = object_server
            .interface::<_, SettingsInterface>(crate::proxy::DESKTOP_PATH)
            .await?;
        let signal_context = iface_ref.signal_context();

        SettingsInterface::setting_changed(signal_context, namespace, key, value).await
    }

    pub async fn try_next(&self) -> Result<(), crate::Error> {
        if let Some(action) = (*self.receiver.lock().await).next().await {
            self.activate(action).await?;
        }
        Ok(())
    }
}

enum Action {
    ReadAll(Vec<String>, oneshot::Sender<HashMap<String, Namespace>>),
    Read(
        String,
        String,
        oneshot::Sender<Result<OwnedValue, PortalError>>,
    ),
}

struct SettingsInterface {
    sender: Arc<Mutex<Sender<Action>>>,
}

impl SettingsInterface {
    pub fn new(sender: Sender<Action>) -> Self {
        Self {
            sender: Arc::new(Mutex::new(sender)),
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Settings")]
impl SettingsInterface {
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        2
    }

    async fn read_all(&self, namespaces: Vec<String>) -> HashMap<String, Namespace> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Settings::ReadAll");

        let (sender, receiver) = futures_channel::oneshot::channel();
        let _ = self
            .sender
            .lock()
            .await
            .send(Action::ReadAll(namespaces, sender))
            .await;

        let response = receiver.await.unwrap_or_default();

        #[cfg(feature = "tracing")]
        tracing::debug!("Settings::ReadAll returned {:#?}", response);
        response
    }

    async fn read(&self, namespace: String, key: String) -> Result<OwnedValue, PortalError> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Settings::Read");

        let (sender, receiver) = futures_channel::oneshot::channel();
        let _ = self
            .sender
            .lock()
            .await
            .send(Action::Read(namespace, key, sender))
            .await;

        let response = receiver.await.unwrap();

        #[cfg(feature = "tracing")]
        tracing::debug!("Settings::Read returned {:#?}", response);
        response
    }

    #[zbus(signal)]
    async fn setting_changed(
        signal_ctxt: &SignalContext<'_>,
        namespace: &str,
        key: &str,
        value: Value<'_>,
    ) -> zbus::Result<()>;
}

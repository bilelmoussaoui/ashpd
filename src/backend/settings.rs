use std::{collections::HashMap, num::NonZeroU32, sync::Arc};

use async_trait::async_trait;
use futures_channel::{
    mpsc::{Receiver, Sender},
    oneshot,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::Mutex;

use crate::{backend::Backend, desktop::settings::Namespace, zvariant::OwnedValue};

#[async_trait]
pub trait SettingsImpl {
    const VERSION: NonZeroU32;

    async fn read_all(&self, namespaces: Vec<String>) -> HashMap<String, Namespace>;

    async fn read(&self, namespace: &str, key: &str) -> OwnedValue;
}

pub struct Settings<T: SettingsImpl> {
    receiver: Arc<Mutex<Receiver<Action>>>,
    imp: T,
}

impl<T: SettingsImpl> Settings<T> {
    pub async fn new(imp: T, backend: &Backend) -> zbus::Result<Self> {
        let (sender, receiver) = futures_channel::mpsc::channel(10);
        let iface = SettingsInterface::new(sender, T::VERSION);
        backend.serve(iface).await?;
        let provider = Self {
            receiver: Arc::new(Mutex::new(receiver)),
            imp,
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

    pub async fn try_next(&self) -> Result<(), crate::Error> {
        if let Some(action) = (*self.receiver.lock().await).next().await {
            self.activate(action).await?;
        }
        Ok(())
    }
}

enum Action {
    ReadAll(Vec<String>, oneshot::Sender<HashMap<String, Namespace>>),
    Read(String, String, oneshot::Sender<OwnedValue>),
}

struct SettingsInterface {
    sender: Arc<Mutex<Sender<Action>>>,
    version: NonZeroU32,
}

impl SettingsInterface {
    pub fn new(sender: Sender<Action>, version: NonZeroU32) -> Self {
        Self {
            sender: Arc::new(Mutex::new(sender)),
            version,
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Settings")]
impl SettingsInterface {
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        self.version.into()
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

        let response = receiver.await.unwrap();

        #[cfg(feature = "tracing")]
        tracing::debug!("Settings::ReadAll returned {:#?}", response);
        response
    }

    async fn read(&self, namespace: String, key: String) -> OwnedValue {
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
}

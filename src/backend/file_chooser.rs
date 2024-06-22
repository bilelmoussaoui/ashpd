use std::sync::Arc;

use async_lock::Mutex;
use async_trait::async_trait;
use futures_channel::{
    mpsc::{Receiver, Sender},
    oneshot,
};
use futures_util::{SinkExt, StreamExt};

use crate::{
    backend::{
        request::{Request, RequestImpl},
        Backend,
    },
    desktop::{
        file_chooser::{Choice, FileFilter},
        Response,
    },
    zvariant::{DeserializeDict, OwnedObjectPath, SerializeDict, Type},
    AppID, FilePath, WindowIdentifierType,
};

// Does not coincide with the one in desktop/file_chooser.rs
#[derive(DeserializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct OpenFileOptions {
    pub accept_label: Option<String>,
    pub modal: Option<bool>,
    pub multiple: Option<bool>,
    pub directory: Option<bool>,
    pub filters: Option<Vec<FileFilter>>,
    pub current_filter: Option<FileFilter>,
    pub choices: Option<Vec<Choice>>,
}

// Does not coincide with the one in desktop/file_chooser.rs
#[derive(DeserializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct SaveFileOptions {
    pub accept_label: Option<String>,
    pub modal: Option<bool>,
    pub multiple: Option<bool>,
    pub filters: Option<Vec<FileFilter>>,
    pub current_filter: Option<FileFilter>,
    pub choices: Option<Vec<Choice>>,
    pub current_name: Option<String>,
    pub current_folder: Option<FilePath>,
    pub current_file: Option<FilePath>,
}

#[derive(DeserializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct SaveFilesOptions {
    // TODO Its in the xdp docs, but is it correct? See
    // https://github.com/flatpak/xdg-desktop-portal/issues/938
    // pub handle_token: Option<String>,
    pub accept_label: Option<String>,
    pub modal: Option<bool>,
    pub choices: Option<Vec<Choice>>,
    pub current_folder: Option<FilePath>,
    pub files: Option<Vec<FilePath>>,
}

#[derive(DeserializeDict, SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct OpenFileResults {
    pub uris: Option<Vec<url::Url>>,
    pub choices: Option<Vec<Choice>>,
    pub current_filter: Option<FileFilter>,
    pub writable: Option<bool>,
}

#[derive(DeserializeDict, SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct SaveFileResults {
    pub uris: Option<Vec<url::Url>>,
    pub choices: Option<Vec<Choice>>,
    pub current_filter: Option<FileFilter>,
}

#[derive(DeserializeDict, SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct SaveFilesResults {
    pub uris: Option<Vec<url::Url>>,
    pub choices: Option<Vec<Choice>>,
}

#[async_trait]
pub trait FileChooserImpl {
    async fn open_file(
        &self,
        app_id: AppID,
        window_identifier: WindowIdentifierType,
        title: &str,
        options: OpenFileOptions,
    ) -> Response<OpenFileResults>;

    async fn save_file(
        &self,
        app_id: AppID,
        window_identifier: WindowIdentifierType,
        title: &str,
        options: SaveFileOptions,
    ) -> Response<SaveFileResults>;

    async fn save_files(
        &self,
        app_id: AppID,
        window_identifier: WindowIdentifierType,
        title: &str,
        options: SaveFilesOptions,
    ) -> Response<SaveFilesResults>;
}

pub struct FileChooser<T: FileChooserImpl + RequestImpl> {
    receiver: Arc<Mutex<Receiver<Action>>>,
    imp: Arc<T>,
    cnx: zbus::Connection,
}

impl<T: FileChooserImpl + RequestImpl> FileChooser<T> {
    pub async fn new(imp: T, backend: &Backend) -> zbus::Result<Self> {
        let (sender, receiver) = futures_channel::mpsc::channel(10);
        let iface = FileChooserInterface::new(sender);
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
            Action::OpenFile(path, app_id, window_identifier, title, options, sender) => {
                let request = Request::new(Arc::clone(&self.imp), path, &self.cnx).await?;
                let results = self
                    .imp
                    .open_file(app_id, window_identifier, &title, options)
                    .await;
                let _ = sender.send(results);
                request.next().await?;
            }
            Action::SaveFile(path, app_id, window_identifier, title, options, sender) => {
                let request = Request::new(Arc::clone(&self.imp), path, &self.cnx).await?;
                let results = self
                    .imp
                    .save_file(app_id, window_identifier, &title, options)
                    .await;
                let _ = sender.send(results);
                request.next().await?;
            }
            Action::SaveFiles(path, app_id, window_identifier, title, options, sender) => {
                let request = Request::new(Arc::clone(&self.imp), path, &self.cnx).await?;
                let results = self
                    .imp
                    .save_files(app_id, window_identifier, &title, options)
                    .await;
                let _ = sender.send(results);
                request.next().await?;
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
    OpenFile(
        OwnedObjectPath,
        AppID,
        WindowIdentifierType,
        String,
        OpenFileOptions,
        oneshot::Sender<Response<OpenFileResults>>,
    ),
    SaveFile(
        OwnedObjectPath,
        AppID,
        WindowIdentifierType,
        String,
        SaveFileOptions,
        oneshot::Sender<Response<SaveFileResults>>,
    ),
    SaveFiles(
        OwnedObjectPath,
        AppID,
        WindowIdentifierType,
        String,
        SaveFilesOptions,
        oneshot::Sender<Response<SaveFilesResults>>,
    ),
}
struct FileChooserInterface {
    sender: Arc<Mutex<Sender<Action>>>,
}

impl FileChooserInterface {
    pub fn new(sender: Sender<Action>) -> Self {
        Self {
            sender: Arc::new(Mutex::new(sender)),
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.FileChooser")]
impl FileChooserInterface {
    async fn open_file(
        &self,
        handle: OwnedObjectPath,
        app_id: AppID,
        window_identifier: WindowIdentifierType,
        title: String,
        options: OpenFileOptions,
    ) -> Response<OpenFileResults> {
        let (sender, receiver) = futures_channel::oneshot::channel();
        let _ = self
            .sender
            .lock()
            .await
            .send(Action::OpenFile(
                handle,
                app_id,
                window_identifier,
                title,
                options,
                sender,
            ))
            .await;

        receiver.await.unwrap()
    }

    async fn save_file(
        &self,
        handle: OwnedObjectPath,
        app_id: AppID,
        window_identifier: WindowIdentifierType,
        title: String,
        options: SaveFileOptions,
    ) -> Response<SaveFileResults> {
        let (sender, receiver) = futures_channel::oneshot::channel();
        let _ = self
            .sender
            .lock()
            .await
            .send(Action::SaveFile(
                handle,
                app_id,
                window_identifier,
                title,
                options,
                sender,
            ))
            .await;

        receiver.await.unwrap()
    }

    async fn save_files(
        &self,
        handle: OwnedObjectPath,
        app_id: AppID,
        window_identifier: WindowIdentifierType,
        title: String,
        options: SaveFilesOptions,
    ) -> Response<SaveFilesResults> {
        let (sender, receiver) = futures_channel::oneshot::channel();
        let _ = self
            .sender
            .lock()
            .await
            .send(Action::SaveFiles(
                handle,
                app_id,
                window_identifier,
                title,
                options,
                sender,
            ))
            .await;

        receiver.await.unwrap()
    }
}

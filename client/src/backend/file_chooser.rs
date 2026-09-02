use std::sync::Arc;

use async_trait::async_trait;
use zbus::message::Header;

use crate::{
    MaybeAppID, WindowIdentifierType,
    backend::{
        Result,
        caller::CallerAuthorization,
        request::{Request, RequestImpl},
    },
    desktop::{
        HandleToken,
        file_chooser::{OpenFileOptions, SaveFileOptions, SaveFilesOptions, SelectedFiles},
        request::Response,
    },
    zvariant::{Optional, OwnedObjectPath},
};

#[async_trait]
pub trait FileChooserImpl: RequestImpl {
    #[doc(alias = "OpenFile")]
    async fn open_file(
        &self,
        token: HandleToken,
        app_id: Option<MaybeAppID>,
        window_identifier: Option<WindowIdentifierType>,
        title: &str,
        options: OpenFileOptions,
    ) -> Result<SelectedFiles>;

    #[doc(alias = "SaveFile")]
    async fn save_file(
        &self,
        token: HandleToken,
        app_id: Option<MaybeAppID>,
        window_identifier: Option<WindowIdentifierType>,
        title: &str,
        options: SaveFileOptions,
    ) -> Result<SelectedFiles>;

    #[doc(alias = "SaveFiles")]
    async fn save_files(
        &self,
        token: HandleToken,
        app_id: Option<MaybeAppID>,
        window_identifier: Option<WindowIdentifierType>,
        title: &str,
        options: SaveFilesOptions,
    ) -> Result<SelectedFiles>;
}

pub(crate) struct FileChooserInterface {
    imp: Arc<dyn FileChooserImpl>,
    spawn: Arc<dyn futures_util::task::Spawn + Send + Sync>,
    cnx: zbus::Connection,
    caller_authorization: Arc<CallerAuthorization>,
}

impl FileChooserInterface {
    pub fn new(
        imp: Arc<dyn FileChooserImpl>,
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

#[zbus::interface(name = "org.freedesktop.impl.portal.FileChooser")]
impl FileChooserInterface {
    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        4
    }

    #[zbus(out_args("response", "results"))]
    async fn open_file(
        &self,
        handle: OwnedObjectPath,
        app_id: Optional<MaybeAppID>,
        window_identifier: Optional<WindowIdentifierType>,
        title: String,
        options: OpenFileOptions,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<Response<SelectedFiles>> {
        self.caller_authorization
            .authorize(&self.cnx, &header)
            .await?;
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "FileChooser::OpenFile",
            &self.cnx,
            Arc::clone(&self.caller_authorization),
            handle.clone(),
            Arc::clone(&self.imp),
            Arc::clone(&self.spawn),
            async move {
                imp.open_file(
                    HandleToken::try_from(&handle).unwrap(),
                    app_id.into(),
                    window_identifier.into(),
                    &title,
                    options,
                )
                .await
            },
        )
        .await
    }

    #[zbus(out_args("response", "results"))]
    async fn save_file(
        &self,
        handle: OwnedObjectPath,
        app_id: Optional<MaybeAppID>,
        window_identifier: Optional<WindowIdentifierType>,
        title: String,
        options: SaveFileOptions,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<Response<SelectedFiles>> {
        self.caller_authorization
            .authorize(&self.cnx, &header)
            .await?;
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "FileChooser::SaveFile",
            &self.cnx,
            Arc::clone(&self.caller_authorization),
            handle.clone(),
            Arc::clone(&self.imp),
            Arc::clone(&self.spawn),
            async move {
                imp.save_file(
                    HandleToken::try_from(&handle).unwrap(),
                    app_id.into(),
                    window_identifier.into(),
                    &title,
                    options,
                )
                .await
            },
        )
        .await
    }

    #[zbus(out_args("response", "results"))]
    async fn save_files(
        &self,
        handle: OwnedObjectPath,
        app_id: Optional<MaybeAppID>,
        window_identifier: Optional<WindowIdentifierType>,
        title: String,
        options: SaveFilesOptions,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<Response<SelectedFiles>> {
        self.caller_authorization
            .authorize(&self.cnx, &header)
            .await?;
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "FileChooser::SaveFiles",
            &self.cnx,
            Arc::clone(&self.caller_authorization),
            handle.clone(),
            Arc::clone(&self.imp),
            Arc::clone(&self.spawn),
            async move {
                imp.save_files(
                    HandleToken::try_from(&handle).unwrap(),
                    app_id.into(),
                    window_identifier.into(),
                    &title,
                    options,
                )
                .await
            },
        )
        .await
    }
}

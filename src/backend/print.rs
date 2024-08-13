use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    backend::request::{Request, RequestImpl},
    desktop::{
        print::{PageSetup, PreparePrint, Settings},
        request::Response,
    },
    zvariant::{self, DeserializeDict, OwnedObjectPath},
    AppID, WindowIdentifierType,
};

#[derive(DeserializeDict, zvariant::Type)]
#[zvariant(signature = "dict")]
pub struct PreparePrintOptions {
    modal: Option<bool>,
    accept_label: Option<String>,
}

impl PreparePrintOptions {
    pub fn is_modal(&self) -> Option<bool> {
        self.modal
    }

    pub fn accept_label(&self) -> Option<&str> {
        self.accept_label.as_deref()
    }
}

#[derive(DeserializeDict, zvariant::Type)]
#[zvariant(signature = "dict")]
pub struct PrintOptions {
    modal: Option<bool>,
    token: Option<u32>,
}

impl PrintOptions {
    pub fn is_modal(&self) -> Option<bool> {
        self.modal
    }

    pub fn token(&self) -> Option<u32> {
        self.token
    }
}

#[async_trait]
pub trait PrintImpl: RequestImpl {
    async fn prepare_print(
        &self,
        app_id: Option<AppID>,
        parent_window: Option<WindowIdentifierType>,
        title: String,
        settings: Settings,
        page_setup: PageSetup,
        options: PreparePrintOptions,
    ) -> Response<PreparePrint>;

    async fn print(
        &self,
        app_id: Option<AppID>,
        parent_window: Option<WindowIdentifierType>,
        title: String,
        fd: zvariant::OwnedFd,
        options: PrintOptions,
    ) -> Response<()>;
}

pub struct PrintInterface {
    imp: Arc<dyn PrintImpl>,
    cnx: zbus::Connection,
}

impl PrintInterface {
    pub fn new(imp: impl PrintImpl + 'static, cnx: zbus::Connection) -> Self {
        Self {
            imp: Arc::new(imp),
            cnx,
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Print")]
impl PrintInterface {
    #[dbus_interface(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        3
    }

    #[allow(clippy::too_many_arguments)]
    #[dbus_interface(out_args("response", "results"))]
    async fn prepare_print(
        &self,
        handle: OwnedObjectPath,
        app_id: &str,
        parent_window: &str,
        title: String,
        settings: Settings,
        page_setup: PageSetup,
        options: PreparePrintOptions,
    ) -> Response<PreparePrint> {
        let window_identifier = WindowIdentifierType::from_maybe_str(parent_window);
        let app_id = AppID::from_maybe_str(app_id);
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Print::PreparePrint",
            &self.cnx,
            handle,
            Arc::clone(&self.imp),
            async move {
                imp.prepare_print(
                    app_id,
                    window_identifier,
                    title,
                    settings,
                    page_setup,
                    options,
                )
                .await
            },
        )
        .await
        .unwrap_or(Response::other())
    }

    #[allow(clippy::too_many_arguments)]
    #[dbus_interface(out_args("response", "results"))]
    async fn print(
        &self,
        handle: OwnedObjectPath,
        app_id: &str,
        parent_window: &str,
        title: String,
        fd: zvariant::OwnedFd,
        options: PrintOptions,
    ) -> Response<()> {
        let window_identifier = WindowIdentifierType::from_maybe_str(parent_window);
        let app_id = AppID::from_maybe_str(app_id);
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Print::Print",
            &self.cnx,
            handle,
            Arc::clone(&self.imp),
            async move {
                imp.print(app_id, window_identifier, title, fd, options)
                    .await
            },
        )
        .await
        .unwrap_or(Response::other())
    }
}
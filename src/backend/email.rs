use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    backend::request::{Request, RequestImpl},
    desktop::request::Response,
    zvariant::{self, DeserializeDict, OwnedObjectPath},
    ActivationToken, AppID, WindowIdentifierType,
};

#[derive(DeserializeDict, zvariant::Type)]
#[zvariant(signature = "dict")]
pub struct Options {
    address: Option<String>,
    addresses: Option<Vec<String>>,
    cc: Option<Vec<String>>,
    bcc: Option<Vec<String>>,
    subject: Option<String>,
    body: Option<String>,
    attachments: Option<Vec<url::Url>>,
    activation_token: Option<ActivationToken>,
}

impl Options {
    pub fn address(&self) -> Option<&str> {
        self.address.as_deref()
    }

    pub fn addresses(&self) -> &[String] {
        self.addresses.as_deref().unwrap_or_default()
    }

    pub fn cc(&self) -> &[String] {
        self.cc.as_deref().unwrap_or_default()
    }

    pub fn bcc(&self) -> &[String] {
        self.bcc.as_deref().unwrap_or_default()
    }

    pub fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }

    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    pub fn attachments(&self) -> &[url::Url] {
        self.attachments.as_deref().unwrap_or_default()
    }

    pub fn activation_token(&self) -> Option<&ActivationToken> {
        self.activation_token.as_ref()
    }
}

#[async_trait]
pub trait EmailImpl: RequestImpl {
    async fn compose(
        &self,
        app_id: Option<AppID>,
        window_identifier: Option<WindowIdentifierType>,
        options: Options,
    ) -> Response<()>;
}

pub struct EmailInterface {
    imp: Arc<dyn EmailImpl>,
    cnx: zbus::Connection,
}

impl EmailInterface {
    pub fn new(imp: impl EmailImpl + 'static, cnx: zbus::Connection) -> Self {
        Self {
            imp: Arc::new(imp),
            cnx,
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Email")]
impl EmailInterface {
    #[dbus_interface(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        4
    }

    #[dbus_interface(out_args("response", "results"))]
    async fn compose_email(
        &self,
        handle: OwnedObjectPath,
        app_id: &str,
        parent_window: &str,
        options: Options,
    ) -> Response<()> {
        let window_identifier = WindowIdentifierType::from_maybe_str(parent_window);
        let app_id = AppID::from_maybe_str(app_id);
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Email::ComposeEmail",
            &self.cnx,
            handle,
            Arc::clone(&self.imp),
            async move { imp.compose(app_id, window_identifier, options).await },
        )
        .await
        .unwrap_or(Response::other())
    }
}

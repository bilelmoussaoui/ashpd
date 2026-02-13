use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    ActivationToken, AppID, Uri, WindowIdentifierType,
    backend::{
        Result,
        request::{Request, RequestImpl},
    },
    desktop::{HandleToken, request::Response},
    zvariant::{self, DeserializeDict, Optional, OwnedObjectPath},
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
    attachments: Option<Vec<Uri>>,
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

    pub fn attachments(&self) -> &[Uri] {
        self.attachments.as_deref().unwrap_or_default()
    }

    pub fn activation_token(&self) -> Option<&ActivationToken> {
        self.activation_token.as_ref()
    }
}

#[async_trait]
pub trait EmailImpl: RequestImpl {
    #[doc(alias = "ComposeEmail")]
    async fn compose(
        &self,
        token: HandleToken,
        app_id: Option<AppID>,
        window_identifier: Option<WindowIdentifierType>,
        options: Options,
    ) -> Result<()>;
}

pub(crate) struct EmailInterface {
    imp: Arc<dyn EmailImpl>,
    spawn: Arc<dyn futures_util::task::Spawn + Send + Sync>,
    cnx: zbus::Connection,
}

impl EmailInterface {
    pub fn new(
        imp: Arc<dyn EmailImpl>,
        cnx: zbus::Connection,
        spawn: Arc<dyn futures_util::task::Spawn + Send + Sync>,
    ) -> Self {
        Self { imp, cnx, spawn }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Email")]
impl EmailInterface {
    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        4
    }

    #[zbus(out_args("response", "results"))]
    async fn compose_email(
        &self,
        handle: OwnedObjectPath,
        app_id: Optional<AppID>,
        window_identifier: Optional<WindowIdentifierType>,
        options: Options,
    ) -> Result<Response<()>> {
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Email::ComposeEmail",
            &self.cnx,
            handle.clone(),
            Arc::clone(&self.imp),
            Arc::clone(&self.spawn),
            async move {
                imp.compose(
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

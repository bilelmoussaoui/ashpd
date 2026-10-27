use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    MaybeAppID, WindowIdentifierType,
    backend::{
        IfaceInner, Result,
        request::{Request, RequestImpl},
    },
    desktop::{HandleToken, email::EmailOptions, request::Response},
    wire::{Optional, OwnedObjectPath},
};

#[async_trait]
pub trait EmailImpl: RequestImpl {
    #[doc(alias = "ComposeEmail")]
    async fn compose(
        &self,
        token: HandleToken,
        app_id: Option<MaybeAppID>,
        window_identifier: Option<WindowIdentifierType>,
        options: EmailOptions,
    ) -> Result<()>;
}

pub(crate) struct EmailInterface {
    imp: Arc<dyn EmailImpl>,
    inner: IfaceInner,
}

impl EmailInterface {
    pub fn new(imp: Arc<dyn EmailImpl>, inner: IfaceInner) -> Self {
        Self { imp, inner }
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
        app_id: Optional<MaybeAppID>,
        window_identifier: Optional<WindowIdentifierType>,
        options: EmailOptions,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) -> Result<Response<()>> {
        self.inner
            .caller_authorization
            .authorize(&self.inner.cnx, &header)
            .await?;
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Email::ComposeEmail",
            &self.inner,
            handle.clone(),
            Arc::clone(&self.imp),
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

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
    desktop::{HandleToken, email::EmailOptions, request::Response},
    zvariant::{Optional, OwnedObjectPath},
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
    spawn: Arc<dyn futures_util::task::Spawn + Send + Sync>,
    cnx: zbus::Connection,
    caller_authorization: Arc<CallerAuthorization>,
}

impl EmailInterface {
    pub fn new(
        imp: Arc<dyn EmailImpl>,
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
        #[zbus(header)] header: Header<'_>,
    ) -> Result<Response<()>> {
        self.caller_authorization
            .authorize(&self.cnx, &header)
            .await?;
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Email::ComposeEmail",
            &self.cnx,
            Arc::clone(&self.caller_authorization),
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

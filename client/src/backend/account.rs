use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    MaybeAppID, WindowIdentifierType,
    backend::{
        IfaceInner, Result,
        request::{Request, RequestImpl},
    },
    desktop::{
        HandleToken,
        account::{UserInformation, UserInformationOptions},
        request::Response,
    },
    wire::{Optional, OwnedObjectPath},
};

#[async_trait]
pub trait AccountImpl: RequestImpl {
    #[doc(alias = "GetUserInformation")]
    async fn get_user_information(
        &self,
        token: HandleToken,
        app_id: Option<MaybeAppID>,
        window_identifier: Option<WindowIdentifierType>,
        options: UserInformationOptions,
    ) -> Result<UserInformation>;
}

pub(crate) struct AccountInterface {
    imp: Arc<dyn AccountImpl>,
    inner: IfaceInner,
}

impl AccountInterface {
    pub fn new(imp: Arc<dyn AccountImpl>, inner: IfaceInner) -> Self {
        Self { imp, inner }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Account")]
impl AccountInterface {
    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        1
    }

    #[zbus(name = "GetUserInformation")]
    #[zbus(out_args("response", "results"))]
    async fn get_user_information(
        &self,
        handle: OwnedObjectPath,
        app_id: Optional<MaybeAppID>,
        window_identifier: Optional<WindowIdentifierType>,
        options: UserInformationOptions,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) -> Result<Response<UserInformation>> {
        self.inner
            .caller_authorization
            .authorize(&self.inner.cnx, &header)
            .await?;
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Account::GetUserInformation",
            &self.inner,
            handle.clone(),
            Arc::clone(&self.imp),
            async move {
                imp.get_user_information(
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

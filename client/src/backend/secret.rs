use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use zbus::wire::{self, OwnedVariant};

use crate::{
    MaybeAppID,
    backend::{
        IfaceInner, Result,
        request::{Request, RequestImpl},
    },
    desktop::{HandleToken, Response},
};

#[async_trait]
pub trait SecretImpl: RequestImpl {
    /// Retrieve a secret for `app_id`.
    ///
    /// The D-Bus interface applies the caller authorization configured on
    /// [`crate::backend::Builder`] before invoking this method.
    #[doc(alias = "RetrieveSecret")]
    async fn retrieve(
        &self,
        token: HandleToken,
        app_id: MaybeAppID,
        fd: std::os::fd::OwnedFd,
    ) -> Result<HashMap<String, OwnedVariant>>;
}

pub(crate) struct SecretInterface {
    imp: Arc<dyn SecretImpl>,
    inner: IfaceInner,
}

impl SecretInterface {
    pub fn new(imp: Arc<dyn SecretImpl>, inner: IfaceInner) -> Self {
        Self { imp, inner }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Secret")]
impl SecretInterface {
    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        1
    }

    #[zbus(out_args("response", "results"))]
    async fn retrieve_secret(
        &self,
        handle: wire::OwnedObjectPath,
        app_id: MaybeAppID,
        fd: wire::OwnedFd,
        _options: HashMap<String, OwnedVariant>,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) -> Result<Response<HashMap<String, OwnedVariant>>> {
        self.inner
            .caller_authorization
            .authorize(&self.inner.cnx, &header)
            .await?;
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Secret::RetrieveSecret",
            &self.inner,
            handle.clone(),
            Arc::clone(&self.imp),
            async move {
                imp.retrieve(
                    HandleToken::try_from(&handle).unwrap(),
                    app_id,
                    std::os::fd::OwnedFd::from(fd),
                )
                .await
            },
        )
        .await
    }
}

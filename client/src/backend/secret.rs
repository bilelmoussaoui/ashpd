use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use zbus::{
    message::Header,
    zvariant::{self, OwnedValue},
};

use crate::{
    MaybeAppID,
    backend::{
        Result,
        caller::CallerAuthorization,
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
    ) -> Result<HashMap<String, OwnedValue>>;
}

pub(crate) struct SecretInterface {
    imp: Arc<dyn SecretImpl>,
    spawn: Arc<dyn futures_util::task::Spawn + Send + Sync>,
    cnx: zbus::Connection,
    caller_authorization: Arc<CallerAuthorization>,
}

impl SecretInterface {
    pub fn new(
        imp: Arc<dyn SecretImpl>,
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

#[zbus::interface(name = "org.freedesktop.impl.portal.Secret")]
impl SecretInterface {
    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        1
    }

    #[zbus(out_args("response", "results"))]
    async fn retrieve_secret(
        &self,
        handle: zvariant::OwnedObjectPath,
        app_id: MaybeAppID,
        fd: zvariant::OwnedFd,
        _options: HashMap<String, OwnedValue>,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<Response<HashMap<String, OwnedValue>>> {
        self.caller_authorization
            .authorize(&self.cnx, &header)
            .await?;
        let imp = Arc::clone(&self.imp);

        Request::spawn(
            "Secret::RetrieveSecret",
            &self.cnx,
            Arc::clone(&self.caller_authorization),
            handle.clone(),
            Arc::clone(&self.imp),
            Arc::clone(&self.spawn),
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

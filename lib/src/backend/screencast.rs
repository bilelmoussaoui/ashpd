use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use zbus::zvariant::OwnedValue;

use crate::{
    backend::{
        request::{Request, RequestImpl},
        session::{Session, SessionImpl, CreateSessionResponse},
        MaybeAppID, MaybeWindowIdentifier, Result,
    },
    desktop::request::Response,
    zvariant::{DeserializeDict, OwnedObjectPath, Type},
    AppID, WindowIdentifierType,
};

#[async_trait]
pub trait ScreenCastImpl: RequestImpl + SessionImpl {
    async fn create_session(
        &self,
        app_id: Option<AppID>,
        options: HashMap<String, OwnedValue>,
        session: Session,
    ) -> Result<()>;
}

pub struct ScreenCastInterface {
    imp: Arc<dyn ScreenCastImpl>,
    session: Option<Arc<dyn SessionImpl>>,
    cnx: zbus::Connection,
}

impl ScreenCastInterface {
    pub fn new(imp: Arc<dyn ScreenCastImpl>, cnx: zbus::Connection) -> Self {
        Self { imp, cnx }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.ScreenCast")]
impl ScreenCastInterface {
    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        5
    }

    #[zbus(name = "CreateSession")]
    #[zbus(out_args("response", "results"))]
    pub async fn create_session(
        &self,
        handle: OwnedObjectPath,
        session_handle: OwnedObjectPath,
        app_id: MaybeAppID,
        options: HashMap<String, OwnedValue>,
    ) -> Result<Response<CreateSessionResponse>> {
        let imp = Arc::clone(&self.imp);
        let cnx = self.cnx.clone();
        Request::spawn(
            "ScreenCast::CreateSession",
            &self.cnx,
            handle,
            Arc::clone(&self.imp),
            async move {
                imp.create_session(app_id.inner(), options).await?;
                let session = Session::spawn(
                    "ScreenCast::CreateSession",
                    &cnx,
                    session_handle,
                    Arc::clone(&imp),
                )
                .await?;
            },
        )
        .await
    }
    // #[zbus(name = "SelectSources")]
    // #[zbus(out_args("response", "results"))]
    // pub async fn select_sources(
    // &self,
    // handle: OwnedObjectPath,
    // session_handle: OwnedObjectPath,
    // app_id: MaybeAppID,
    // options: SelectSourcesOptions,
    // ) -> Result<Response<()>> {
    // Ok(Response::other())
    // }
    //
    // #[zbus(name = "Start")]
    // #[zbus(out_args("response", "results"))]
    // pub async fn start(
    // &self,
    // handle: OwnedObjectPath,
    // session_handle: OwnedObjectPath,
    // app_id: MaybeAppID,
    // parent_window: MaybeWindowIdentifier,
    // _options: HashMap<String, OwnedValue>,
    // ) -> Result<Response<()>> {
    // Ok(Response::other())
    // }
}

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::Serialize;
use zbus::message::Header;

use crate::{
    MaybeAppID, PortalError, WindowIdentifierType,
    backend::{
        IfaceInner, Result,
        request::{Request, RequestImpl},
        session::{CreateSessionResponse, Session, SessionImpl, SessionManager},
    },
    desktop::{
        CreateSessionOptions, HandleToken,
        request::Response,
        screencast::{CursorMode, SelectSourcesOptions, SourceType, StartCastOptions, Streams},
    },
    wire::{Optional, OwnedObjectPath, Type},
};

#[derive(Serialize, Type, Debug, Default)]
#[zbus(signature = "dict")]
pub struct SelectSourcesResponse {}

#[async_trait]
pub trait ScreencastImpl: RequestImpl + SessionImpl {
    #[doc(alias = "AvailableSourceTypes")]
    fn available_source_types(&self) -> SourceType;

    #[doc(alias = "AvailableCursorModes")]
    fn available_cursor_mode(&self) -> CursorMode;

    #[doc(alias = "CreateSession")]
    async fn create_session(
        &self,
        token: HandleToken,
        session_token: HandleToken,
        app_id: Option<MaybeAppID>,
        options: CreateSessionOptions,
    ) -> Result<CreateSessionResponse>;

    #[doc(alias = "SelectSources")]
    async fn select_sources(
        &self,
        token: HandleToken,
        session_token: HandleToken,
        app_id: Option<MaybeAppID>,
        options: SelectSourcesOptions,
    ) -> Result<SelectSourcesResponse>;

    #[doc(alias = "Start")]
    async fn start_cast(
        &self,
        token: HandleToken,
        session_token: HandleToken,
        app_id: Option<MaybeAppID>,
        window_identifier: Option<WindowIdentifierType>,
        options: StartCastOptions,
    ) -> Result<Streams>;
}

pub(crate) struct ScreencastInterface {
    imp: Arc<dyn ScreencastImpl>,
    inner: IfaceInner,
    sessions: Arc<Mutex<SessionManager>>,
}

impl ScreencastInterface {
    pub fn new(
        imp: Arc<dyn ScreencastImpl>,
        inner: IfaceInner,
        sessions: Arc<Mutex<SessionManager>>,
    ) -> Self {
        Self {
            imp,
            inner,
            sessions,
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.ScreenCast")]
impl ScreencastInterface {
    #[zbus(
        property(emits_changed_signal = "const"),
        name = "AvailableSourceTypes"
    )]
    fn available_source_types(&self) -> SourceType {
        let imp = Arc::clone(&self.imp);
        imp.available_source_types()
    }

    #[zbus(
        property(emits_changed_signal = "const"),
        name = "AvailableCursorModes"
    )]
    fn available_cursor_mode(&self) -> CursorMode {
        let imp = Arc::clone(&self.imp);
        imp.available_cursor_mode()
    }

    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        6
    }

    #[zbus(name = "CreateSession")]
    #[zbus(out_args("response", "results"))]
    async fn create_session(
        &self,
        handle: OwnedObjectPath,
        session_handle: OwnedObjectPath,
        app_id: Optional<MaybeAppID>,
        options: CreateSessionOptions,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<Response<CreateSessionResponse>> {
        self.inner
            .caller_authorization
            .authorize(&self.inner.cnx, &header)
            .await?;
        let session_token = HandleToken::try_from(&session_handle).unwrap();
        {
            let sessions = self.sessions.lock().unwrap();
            if sessions.contains(&session_token) {
                let errormsg = format!("A session with handle `{session_token}` already exists");
                #[cfg(feature = "tracing")]
                tracing::error!("ScreencastInterface::create_session: {}", errormsg);
                return Err(PortalError::Exist(errormsg));
            }
        }

        let imp = Arc::clone(&self.imp);
        let token = session_token.clone();
        let result = Request::spawn(
            "ScreenCast::CreateSession",
            &self.inner,
            handle.clone(),
            Arc::clone(&self.imp),
            async move {
                imp.create_session(
                    HandleToken::try_from(&handle).unwrap(),
                    token,
                    app_id.into(),
                    options,
                )
                .await
            },
        )
        .await;

        if result.is_ok() {
            #[cfg(feature = "tracing")]
            tracing::debug!(
                "ScreencastInterface::create_session: session with handle `{session_token}` created"
            );

            let session = Session::new(
                session_handle,
                Arc::clone(&self.sessions),
                Some(Arc::clone(&self.imp) as Arc<dyn SessionImpl>),
                Arc::clone(&self.inner.caller_authorization),
            );
            session.serve(self.inner.cnx.clone()).await?;
            {
                let mut sessions = self.sessions.lock().unwrap();
                sessions.add(session);
            }
        } else {
            #[cfg(feature = "tracing")]
            tracing::error!(
                "ScreencastInterface::create_session: failed to create a session with handle `{session_token}`"
            );
        }

        result
    }

    #[zbus(name = "SelectSources")]
    #[zbus(out_args("response", "results"))]
    async fn select_sources(
        &self,
        handle: OwnedObjectPath,
        session_handle: OwnedObjectPath,
        app_id: Optional<MaybeAppID>,
        options: SelectSourcesOptions,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<Response<SelectSourcesResponse>> {
        self.inner
            .caller_authorization
            .authorize(&self.inner.cnx, &header)
            .await?;
        let session_token = HandleToken::try_from(&session_handle).unwrap();
        {
            let sessions = self.sessions.lock().unwrap();
            sessions.try_contains(&session_token)?;
        }

        let imp = Arc::clone(&self.imp);
        Request::spawn(
            "ScreenCast::SelectSources",
            &self.inner,
            handle.clone(),
            Arc::clone(&self.imp),
            async move {
                imp.select_sources(
                    HandleToken::try_from(&handle).unwrap(),
                    session_token,
                    app_id.into(),
                    options,
                )
                .await
            },
        )
        .await
    }

    #[zbus(name = "Start")]
    #[zbus(out_args("response", "results"))]
    async fn start(
        &self,
        handle: OwnedObjectPath,
        session_handle: OwnedObjectPath,
        app_id: Optional<MaybeAppID>,
        window_identifier: Optional<WindowIdentifierType>,
        options: StartCastOptions,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<Response<Streams>> {
        self.inner
            .caller_authorization
            .authorize(&self.inner.cnx, &header)
            .await?;
        let session_token = HandleToken::try_from(&session_handle).unwrap();
        {
            let sessions = self.sessions.lock().unwrap();
            sessions.try_contains(&session_token)?;
        }

        let imp = Arc::clone(&self.imp);
        Request::spawn(
            "ScreenCast::Start",
            &self.inner,
            handle.clone(),
            Arc::clone(&self.imp),
            async move {
                imp.start_cast(
                    HandleToken::try_from(&handle).unwrap(),
                    session_token,
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

use std::{
    borrow::Borrow,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use enumflags2::BitFlags;

use crate::{
    AppID, PortalError, WindowIdentifierType,
    backend::{
        MaybeAppID, MaybeWindowIdentifier, Result,
        request::{Request, RequestImpl},
        session::{CreateSessionResponse, Session, SessionImpl, SessionManager},
    },
    desktop::{
        HandleToken, PersistMode,
        request::Response,
        screencast::{CursorMode, SourceType, Stream},
    },
    zvariant::{DeserializeDict, OwnedObjectPath, OwnedValue, SerializeDict, Type, Value},
};

#[derive(DeserializeDict, Type, Debug)]
#[zvariant(signature = "dict")]
pub struct CreateSessionOptions {}

#[derive(DeserializeDict, Type, Debug)]
#[zvariant(signature = "dict")]
pub struct SelectSourcesOptions {
    types: Option<BitFlags<SourceType>>,
    multiple: Option<bool>,
    cursor_mode: Option<CursorMode>,
    restore_data: Option<(String, u32, OwnedValue)>,
    persist_mode: Option<PersistMode>,
}

impl SelectSourcesOptions {
    pub fn types(&self) -> Option<BitFlags<SourceType>> {
        self.types
    }

    pub fn is_multiple(&self) -> Option<bool> {
        self.multiple
    }

    pub fn cursor_mode(&self) -> Option<CursorMode> {
        self.cursor_mode
    }

    pub fn restore_data<'a>(&'a self) -> Option<(&'a str, u32, &'a Value<'a>)> {
        match &self.restore_data {
            Some((key, version, data)) => Some((key.as_str(), *version, data.borrow())),
            None => None,
        }
    }

    pub fn persist_mode(&self) -> Option<PersistMode> {
        self.persist_mode
    }
}

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct SelectSourcesResponse {}

#[derive(DeserializeDict, Type, Debug)]
#[zvariant(signature = "dict")]
pub struct StartCastOptions {}

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct StartCastResponse {
    streams: Vec<Stream>,
    restore_data: Option<(String, u32, OwnedValue)>,
}

impl StartCastResponse {
    /// The session restore token.
    pub fn restore_data(&self) -> Option<&(String, u32, OwnedValue)> {
        self.restore_data.as_ref()
    }

    /// The list of streams.
    pub fn streams(&self) -> &[Stream] {
        &self.streams
    }
}

pub struct StartCastResponseBuilder {
    response: StartCastResponse,
}

impl StartCastResponseBuilder {
    pub fn new(streams: Vec<Stream>) -> Self {
        Self {
            response: StartCastResponse {
                streams,
                restore_data: None,
            },
        }
    }

    pub fn restore_data(mut self, data: Option<(String, u32, impl Into<OwnedValue>)>) -> Self {
        self.response.restore_data = data.map(|(s, u, d)| (s, u, d.into()));
        self
    }

    pub fn build(self) -> StartCastResponse {
        self.response
    }
}

#[async_trait]
pub trait ScreencastImpl: RequestImpl + SessionImpl {
    #[doc(alias = "AvailableSourceTypes")]
    fn available_source_types(&self) -> BitFlags<SourceType>;

    #[doc(alias = "AvailableCursorModes")]
    fn available_cursor_mode(&self) -> BitFlags<CursorMode>;

    #[doc(alias = "CreateSession")]
    async fn create_session(
        &self,
        token: HandleToken,
        session_token: HandleToken,
        app_id: Option<AppID>,
        options: CreateSessionOptions,
    ) -> Result<CreateSessionResponse>;

    #[doc(alias = "SelectSources")]
    async fn select_sources(
        &self,
        session_token: HandleToken,
        app_id: Option<AppID>,
        options: SelectSourcesOptions,
    ) -> Result<SelectSourcesResponse>;

    #[doc(alias = "Start")]
    async fn start_cast(
        &self,
        session_token: HandleToken,
        app_id: Option<AppID>,
        window_identifier: Option<WindowIdentifierType>,
        options: StartCastOptions,
    ) -> Result<StartCastResponse>;
}

pub(crate) struct ScreencastInterface {
    imp: Arc<dyn ScreencastImpl>,
    spawn: Arc<dyn futures_util::task::Spawn + Send + Sync>,
    cnx: zbus::Connection,
    sessions: Arc<Mutex<SessionManager>>,
}

impl ScreencastInterface {
    pub fn new(
        imp: Arc<dyn ScreencastImpl>,
        cnx: zbus::Connection,
        spawn: Arc<dyn futures_util::task::Spawn + Send + Sync>,
        sessions: Arc<Mutex<SessionManager>>,
    ) -> Self {
        Self {
            imp,
            cnx,
            spawn,
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
    fn available_source_types(&self) -> u32 {
        let imp = Arc::clone(&self.imp);
        imp.available_source_types().bits()
    }

    #[zbus(
        property(emits_changed_signal = "const"),
        name = "AvailableCursorModes"
    )]
    fn available_cursor_mode(&self) -> u32 {
        let imp = Arc::clone(&self.imp);
        imp.available_cursor_mode().bits()
    }

    #[zbus(property(emits_changed_signal = "const"), name = "version")]
    fn version(&self) -> u32 {
        5
    }

    #[zbus(name = "CreateSession")]
    #[zbus(out_args("response", "results"))]
    async fn create_session(
        &self,
        handle: OwnedObjectPath,
        session_handle: OwnedObjectPath,
        app_id: MaybeAppID,
        options: CreateSessionOptions,
    ) -> Result<Response<CreateSessionResponse>> {
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
            &self.cnx,
            handle.clone(),
            Arc::clone(&self.imp),
            Arc::clone(&self.spawn),
            async move {
                imp.create_session(
                    HandleToken::try_from(&handle).unwrap(),
                    token,
                    app_id.inner(),
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
            let monitor = Arc::clone(&self.imp) as Arc<dyn SessionImpl>;
            let session = Session::new(session_handle, Arc::clone(&self.sessions), Some(monitor));
            session.serve(self.cnx.clone()).await?;
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
        app_id: MaybeAppID,
        options: SelectSourcesOptions,
    ) -> Result<Response<SelectSourcesResponse>> {
        let session_token = HandleToken::try_from(&session_handle).unwrap();
        {
            let sessions = self.sessions.lock().unwrap();
            sessions.try_contains(&session_token)?;
        }

        let imp = Arc::clone(&self.imp);
        Request::spawn(
            "ScreenCast::SelectSources",
            &self.cnx,
            handle.clone(),
            Arc::clone(&self.imp),
            Arc::clone(&self.spawn),
            async move {
                imp.select_sources(session_token, app_id.inner(), options)
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
        app_id: MaybeAppID,
        window_identifier: MaybeWindowIdentifier,
        options: StartCastOptions,
    ) -> Result<Response<StartCastResponse>> {
        let session_token = HandleToken::try_from(&session_handle).unwrap();
        {
            let sessions = self.sessions.lock().unwrap();
            sessions.try_contains(&session_token)?;
        }

        let imp = Arc::clone(&self.imp);
        Request::spawn(
            "ScreenCast::Start",
            &self.cnx,
            handle.clone(),
            Arc::clone(&self.imp),
            Arc::clone(&self.spawn),
            async move {
                imp.start_cast(
                    session_token,
                    app_id.inner(),
                    window_identifier.inner(),
                    options,
                )
                .await
            },
        )
        .await
    }
}

use ashpd::{
    backend::{
        request::RequestImpl,
        screencast::{
            CreateSessionOptions, ScreencastImpl, SelectSourcesOptions, SelectSourcesResponse,
            StartCastOptions, StartCastResponse, StartCastResponseBuilder,
        },
        session::{CreateSessionResponse, SessionImpl},
        Result,
    },
    desktop::{
        screencast::{CursorMode, SourceType, StreamBuilder},
        HandleToken,
    },
    enumflags2::BitFlags,
    AppID, WindowIdentifierType,
};
use async_trait::async_trait;

#[derive(Default)]
pub struct Screencast {}

#[async_trait]
impl RequestImpl for Screencast {
    async fn close(&self, token: HandleToken) {
        tracing::debug!("IN Close(): {token}");
    }
}

#[async_trait]
impl ScreencastImpl for Screencast {
    fn available_source_types(&self) -> BitFlags<SourceType> {
        SourceType::Monitor | SourceType::Window
    }

    fn available_cursor_mode(&self) -> BitFlags<CursorMode> {
        CursorMode::Hidden | CursorMode::Embedded | CursorMode::Metadata
    }

    async fn create_session(
        &self,
        _token: HandleToken,
        session_token: HandleToken,
        _app_id: Option<AppID>,
        _options: CreateSessionOptions,
    ) -> Result<CreateSessionResponse> {
        tracing::debug!("IN Screencast::create_session(): {session_token}");
        Ok(CreateSessionResponse::new(session_token))
    }

    async fn select_sources(
        &self,
        session_token: HandleToken,
        _app_id: Option<AppID>,
        _options: SelectSourcesOptions,
    ) -> Result<SelectSourcesResponse> {
        tracing::debug!("IN Screencast::select_sources(): {session_token}");
        Ok(SelectSourcesResponse::default())
    }

    async fn start_cast(
        &self,
        session_token: HandleToken,
        _app_id: Option<AppID>,
        _window_identifier: Option<WindowIdentifierType>,
        _options: StartCastOptions,
    ) -> Result<StartCastResponse> {
        tracing::debug!("IN Screencast::start_cast(): {session_token}");
        let streams = vec![StreamBuilder::new(42)
            .source_type(SourceType::Monitor)
            .build()];
        Ok(StartCastResponseBuilder::new(streams).build())
    }
}

#[async_trait]
impl SessionImpl for Screencast {
    async fn session_closed(&self, session_token: HandleToken) -> Result<()> {
        tracing::debug!("IN Screencast::session_closed(): {session_token}");
        Ok(())
    }
}

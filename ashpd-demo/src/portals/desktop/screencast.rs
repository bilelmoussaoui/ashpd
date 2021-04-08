use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::{collections::HashMap, convert::TryFrom, sync::Arc};

use adw::prelude::*;
use ashpd::{
    desktop::screencast::{
        AsyncScreenCastProxy, CreateSession, CreateSessionOptions, CursorMode,
        SelectSourcesOptions, SourceType, StartCastOptions, Stream, Streams,
    },
    enumflags2::BitFlags,
    BasicResponse, HandleToken,
};
use ashpd::{zbus, zvariant};
use ashpd::{AsyncSessionProxy, Response, WindowIdentifier};
use futures::lock::Mutex;
use futures::FutureExt;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::widgets::CameraPaintable;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/screencast.ui")]
    pub struct ScreenCastPage {
        #[template_child]
        pub streams_carousel: TemplateChild<adw::Carousel>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub types_comborow: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub cursor_comborow: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub multiple_switch: TemplateChild<gtk::Switch>,
        pub session: Arc<Mutex<Option<AsyncSessionProxy<'static>>>>,
        #[template_child]
        pub close_session_btn: TemplateChild<gtk::Button>,
    }

    impl Default for ScreenCastPage {
        fn default() -> Self {
            Self {
                response_group: TemplateChild::default(),
                streams_carousel: TemplateChild::default(),
                cursor_comborow: TemplateChild::default(),
                types_comborow: TemplateChild::default(),
                multiple_switch: TemplateChild::default(),
                close_session_btn: TemplateChild::default(),
                session: Arc::new(Mutex::new(None)),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScreenCastPage {
        const NAME: &'static str = "ScreenCastPage";
        type Type = super::ScreenCastPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("screencast.start", None, move |page, _action, _target| {
                page.start_session();
            });
            klass.install_action("screencast.stop", None, move |page, _action, _target| {
                page.stop_session();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for ScreenCastPage {
        fn constructed(&self, _obj: &Self::Type) {
            let model = gtk::StringList::new(&["Monitor", "Window", "Both"]);
            self.types_comborow.set_model(Some(&model));
            let model = gtk::StringList::new(&["Hidden", "Embedded", "Metadata"]);
            self.cursor_comborow.set_model(Some(&model));
            self.close_session_btn.set_sensitive(false);
        }
    }
    impl WidgetImpl for ScreenCastPage {}
    impl BinImpl for ScreenCastPage {}
}

glib::wrapper! {
    pub struct ScreenCastPage(ObjectSubclass<imp::ScreenCastPage>) @extends gtk::Widget, adw::Bin;
}

impl ScreenCastPage {
    pub fn new() -> Self {
        let widget = glib::Object::new(&[]).expect("Failed to create a ScreenCastPage");

        let self_ = imp::ScreenCastPage::from_instance(&widget);
        self_.close_session_btn.set_sensitive(false);
        widget
    }

    pub fn start_session(&self) {
        let self_ = imp::ScreenCastPage::from_instance(self);

        let ctx = glib::MainContext::default();
        let streams_carousel = self_.streams_carousel.get();
        let response_group = self_.response_group.get();
        let close_button = self_.close_session_btn.get();
        let multiple = self_.multiple_switch.get_active();
        let types = match self_.types_comborow.get_selected() {
            0 => BitFlags::<SourceType>::from_flag(SourceType::Monitor),
            1 => BitFlags::<SourceType>::from_flag(SourceType::Window),
            _ => SourceType::Monitor | SourceType::Window,
        };
        let cursor_mode = match self_.cursor_comborow.get_selected() {
            0 => BitFlags::<CursorMode>::from_flag(CursorMode::Hidden),
            1 => BitFlags::<CursorMode>::from_flag(CursorMode::Embedded),
            _ => BitFlags::<CursorMode>::from_flag(CursorMode::Metadata),
        };
        ctx.spawn_local(
            clone!(@weak streams_carousel, @weak response_group, @weak self as page => async move {
                if let Ok((streams, fd, session)) = screencast(multiple, types, cursor_mode).await {
                    streams.iter().for_each(|stream| {
                        let paintable = CameraPaintable::new();
                        let picture = gtk::Picture::new();
                        picture.set_paintable(Some(&paintable));
                        picture.set_size_request(400, 400);
                        paintable.set_pipewire_node_id(fd, stream.pipe_wire_node_id());
                        streams_carousel.append(&picture);
                    });

                    let self_ = imp::ScreenCastPage::from_instance(&page);
                    response_group.show();
                    self_.session.lock().await.replace(session);
                    close_button.set_sensitive(true);
                }
            }),
        );
    }

    pub fn stop_session(&self) {
        let self_ = imp::ScreenCastPage::from_instance(self);
        let ctx = glib::MainContext::default();
        let response_group = self_.response_group.get();
        let close_button = self_.close_session_btn.get();
        let streams_carousel = &self_.streams_carousel.get();
        ctx.spawn_local(
            clone!(@weak streams_carousel, @weak response_group, @weak self as page => async move {
                let self_ = imp::ScreenCastPage::from_instance(&page);
                if let Some(session) = self_.session.lock().await.take() {
                    let _ = session.close().await;
                }
                while let Some(child) = streams_carousel.get_next_sibling() {
                    streams_carousel.remove(&child);
                }
                //paintable.close_pipeline();
                response_group.hide();
                close_button.set_sensitive(false);
            }),
        );
    }
}

pub async fn create_session(
    connection: &zbus::azync::Connection,
    proxy: &AsyncScreenCastProxy<'_>,
) -> zbus::Result<AsyncSessionProxy<'static>> {
    let request = proxy
        .create_session(
            CreateSessionOptions::default()
                .session_handle_token(HandleToken::try_from("handletoken").unwrap()),
        )
        .await?;

    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Arc::new(Mutex::new(Some(sender)));
    let signal_id = request
        .connect_response(move |response: Response<CreateSession>| {
            let s = sender.clone();
            async move {
                if let Some(m) = s.lock().await.take() {
                    let _ = m.send(response);
                }
                Ok(())
            }
            .boxed()
        })
        .await?;

    while request.next_signal().await?.is_some() {}
    request.disconnect_signal(signal_id).await?;

    let create_session = receiver.await.unwrap();
    let create_session = create_session.unwrap();
    let session_handle = create_session.session_handle().to_string();

    let session = AsyncSessionProxy::new_for_owned_path(connection.clone(), session_handle)?;

    Ok(session)
}

pub async fn select_sources(
    session: &AsyncSessionProxy<'_>,
    proxy: &AsyncScreenCastProxy<'_>,
    multiple: bool,
    types: BitFlags<SourceType>,
    cursor_mode: BitFlags<CursorMode>,
) -> zbus::Result<Response<BasicResponse>> {
    let request = proxy
        .select_sources(
            session,
            SelectSourcesOptions::default()
                .multiple(multiple)
                .types(types)
                .cursor_mode(cursor_mode),
        )
        .await?;
    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Arc::new(Mutex::new(Some(sender)));
    let signal_id = request
        .connect_response(move |response: Response<BasicResponse>| {
            let s = sender.clone();
            async move {
                if let Some(m) = s.lock().await.take() {
                    let _ = m.send(response);
                }
                Ok(())
            }
            .boxed()
        })
        .await?;
    while request.next_signal().await?.is_some() {}
    request.disconnect_signal(signal_id).await?;

    let response = receiver.await.unwrap();
    Ok(response)
}

pub async fn start_session(
    session: &AsyncSessionProxy<'_>,
    proxy: &AsyncScreenCastProxy<'_>,
) -> zbus::Result<(Vec<Stream>, zvariant::Fd)> {
    let request = proxy
        .start(
            session,
            WindowIdentifier::default(),
            StartCastOptions::default(),
        )
        .await?;
    let (sender, receiver) = futures::channel::oneshot::channel();

    let sender = Arc::new(Mutex::new(Some(sender)));
    let signal_id = request
        .connect_response(move |response: Response<Streams>| {
            let s = sender.clone();
            async move {
                if let Some(m) = s.lock().await.take() {
                    let _ = m.send(response);
                }
                Ok(())
            }
            .boxed()
        })
        .await?;

    while request.next_signal().await?.is_some() {}
    request.disconnect_signal(signal_id).await?;

    let node_id = proxy.open_pipe_wire_remote(session, HashMap::new()).await?;

    if let Response::Ok(streams) = receiver.await.unwrap() {
        Ok((streams.streams().to_vec(), node_id))
    } else {
        Err(zbus::Error::Unsupported)
    }
}

pub async fn screencast(
    multiple: bool,
    types: BitFlags<SourceType>,
    cursor_mode: BitFlags<CursorMode>,
) -> zbus::Result<(Vec<Stream>, RawFd, AsyncSessionProxy<'static>)> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = AsyncScreenCastProxy::new(&connection)?;
    let session = create_session(&connection, &proxy).await?;
    select_sources(&session, &proxy, multiple, types, cursor_mode).await?;

    let (streams, pipewire_fd) = start_session(&session, &proxy).await?;
    Ok((streams, pipewire_fd.as_raw_fd(), session))
}

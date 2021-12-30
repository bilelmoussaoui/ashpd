use crate::widgets::{
    CameraPaintable, NotificationKind, PortalPage, PortalPageExt, PortalPageImpl,
};
use ashpd::{
    desktop::{
        screencast::{CursorMode, PersistMode, ScreenCastProxy, SourceType, Stream},
        SessionProxy,
    },
    enumflags2::BitFlags,
    zbus, WindowIdentifier,
};
use futures::lock::Mutex;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::os::unix::io::RawFd;
use std::sync::Arc;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/screencast.ui")]
    pub struct ScreenCastPage {
        #[template_child]
        pub streams_carousel: TemplateChild<adw::Carousel>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub multiple_switch: TemplateChild<gtk::Switch>,
        pub session: Arc<Mutex<Option<SessionProxy<'static>>>>,
        #[template_child]
        pub monitor_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub window_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub virtual_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub hidden_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub embedded_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub metadata_check: TemplateChild<gtk::CheckButton>,
        pub session_token: Arc<Mutex<Option<String>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScreenCastPage {
        const NAME: &'static str = "ScreenCastPage";
        type Type = super::ScreenCastPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("screencast.start", None, move |page, _action, _target| {
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak page => async move {
                    page.start_session().await;
                }));
            });
            klass.install_action("screencast.stop", None, move |page, _action, _target| {
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak page => async move {
                    page.stop_session().await;
                    page.send_notification("Screen cast session stopped", NotificationKind::Info);
                }));
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for ScreenCastPage {
        fn constructed(&self, obj: &Self::Type) {
            obj.action_set_enabled("screencast.stop", false);

            self.parent_constructed(obj);
        }
    }
    impl WidgetImpl for ScreenCastPage {
        fn map(&self, widget: &Self::Type) {
            let ctx = glib::MainContext::default();
            ctx.spawn_local(clone!(@weak widget as page => async move {
                let self_ = imp::ScreenCastPage::from_instance(&page);
                if let Ok((cursor_modes, source_types)) = available_types().await {
                    self_.virtual_check.set_sensitive(source_types.contains(SourceType::Virtual));
                    self_.monitor_check.set_sensitive(source_types.contains(SourceType::Monitor));
                    self_.window_check.set_sensitive(source_types.contains(SourceType::Window));
                    self_.hidden_check.set_sensitive(cursor_modes.contains(CursorMode::Hidden));
                    self_.metadata_check.set_sensitive(cursor_modes.contains(CursorMode::Metadata));
                    self_.embedded_check.set_sensitive(cursor_modes.contains(CursorMode::Embedded));
                }
            }));
            self.parent_map(widget);
        }
    }
    impl BinImpl for ScreenCastPage {}
    impl PortalPageImpl for ScreenCastPage {}
}

glib::wrapper! {
    pub struct ScreenCastPage(ObjectSubclass<imp::ScreenCastPage>) @extends gtk::Widget, adw::Bin, PortalPage;
}

impl ScreenCastPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a ScreenCastPage")
    }

    /// Returns the selected SourceType
    fn selected_sources(&self) -> BitFlags<SourceType> {
        let self_ = imp::ScreenCastPage::from_instance(self);
        let mut sources: BitFlags<SourceType> = BitFlags::empty();
        if self_.monitor_check.is_active() {
            sources.insert(SourceType::Monitor);
        }
        if self_.window_check.is_active() {
            sources.insert(SourceType::Window);
        }
        sources
    }

    /// Returns the selected CursorMode
    fn selected_cursor_mode(&self) -> BitFlags<CursorMode> {
        let self_ = imp::ScreenCastPage::from_instance(self);

        let mut cursor_mode: BitFlags<CursorMode> = BitFlags::empty();
        if self_.hidden_check.is_active() {
            cursor_mode.insert(CursorMode::Hidden);
        }
        if self_.embedded_check.is_active() {
            cursor_mode.insert(CursorMode::Embedded);
        }
        if self_.metadata_check.is_active() {
            cursor_mode.insert(CursorMode::Metadata);
        }
        cursor_mode
    }

    async fn start_session(&self) {
        let self_ = imp::ScreenCastPage::from_instance(self);
        self.action_set_enabled("screencast.start", false);
        self.action_set_enabled("screencast.stop", true);

        match self.screencast().await {
            Ok((streams, fd, session)) => {
                self.send_notification(
                    "Screen cast session started successfully",
                    NotificationKind::Success,
                );
                streams.iter().for_each(|stream| {
                    let paintable = CameraPaintable::new();
                    let picture = gtk::Picture::builder()
                        .paintable(&paintable)
                        .hexpand(true)
                        .vexpand(true)
                        .build();
                    paintable.set_pipewire_node_id(fd, stream.pipe_wire_node_id());
                    self_.streams_carousel.append(&picture);
                });

                self_.response_group.show();
                self_.session.lock().await.replace(session);
            }
            Err(err) => {
                tracing::error!("{:#?}", err);
                self.send_notification(
                    "Failed to start a screen cast session",
                    NotificationKind::Error,
                );
                self.stop_session().await;
            }
        };
    }

    async fn stop_session(&self) {
        let self_ = imp::ScreenCastPage::from_instance(self);

        self.action_set_enabled("screencast.start", true);
        self.action_set_enabled("screencast.stop", false);

        if let Some(session) = self_.session.lock().await.take() {
            let _ = session.close().await;
        }
        if let Some(mut child) = self_.streams_carousel.first_child() {
            loop {
                let picture = child.downcast_ref::<gtk::Picture>().unwrap();
                let paintable = picture
                    .paintable()
                    .unwrap()
                    .downcast::<CameraPaintable>()
                    .unwrap();
                paintable.close_pipeline();
                self_.streams_carousel.remove(picture);

                if let Some(next_child) = child.next_sibling() {
                    child = next_child;
                } else {
                    break;
                }
            }
        }

        self_.response_group.hide();
    }

    async fn screencast(&self) -> ashpd::Result<(Vec<Stream>, RawFd, SessionProxy<'static>)> {
        let self_ = imp::ScreenCastPage::from_instance(self);
        let sources = self.selected_sources();
        let cursor_mode = self.selected_cursor_mode();
        let multiple = self_.multiple_switch.is_active();

        let root = self.native().unwrap();

        let identifier = WindowIdentifier::from_native(&root).await;

        let connection = zbus::Connection::session().await?;
        let proxy = ScreenCastProxy::new(&connection).await?;
        let session = proxy.create_session().await?;
        let token = self_.session_token.lock().await;
        proxy
            .select_sources(
                &session,
                cursor_mode,
                sources,
                multiple,
                token.as_deref(),
                PersistMode::ExplicitlyRevoked,
            )
            .await?;
        self.send_notification("Starting a screen cast session", NotificationKind::Info);
        let (streams, token) = proxy.start(&session, &identifier).await?;
        if let Some(t) = token {
            self_.session_token.lock().await.replace(t);
        }
        let fd = proxy.open_pipe_wire_remote(&session).await?;
        Ok((streams, fd, session))
    }
}

pub async fn available_types() -> ashpd::Result<(BitFlags<CursorMode>, BitFlags<SourceType>)> {
    let cnx = zbus::Connection::session().await?;
    let proxy = ScreenCastProxy::new(&cnx).await?;

    let cursor_modes = proxy.available_cursor_modes().await?;
    let source_types = proxy.available_source_types().await?;

    Ok((cursor_modes, source_types))
}

use ashpd::{
    desktop::{
        screencast::{CursorMode, ScreenCastProxy, SourceType, Stream},
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

use crate::widgets::CameraPaintable;

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
        pub close_session_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub monitor_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub window_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub hidden_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub embedded_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub metadata_check: TemplateChild<gtk::CheckButton>,
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
        fn constructed(&self, obj: &Self::Type) {
            obj.action_set_enabled("screencast.stop", false);

            self.close_session_btn.set_sensitive(false);
            self.parent_constructed(obj);
        }
    }
    impl WidgetImpl for ScreenCastPage {}
    impl BinImpl for ScreenCastPage {}
}

glib::wrapper! {
    pub struct ScreenCastPage(ObjectSubclass<imp::ScreenCastPage>) @extends gtk::Widget, adw::Bin;
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

    pub fn start_session(&self) {
        let ctx = glib::MainContext::default();
        ctx.spawn_local(clone!(@weak self as page => async move {
            let self_ = imp::ScreenCastPage::from_instance(&page);
            let sources = page.selected_sources();
            let cursor_mode = page.selected_cursor_mode();
            let multiple = self_.multiple_switch.is_active();

            let root = page.native().unwrap();
            page.action_set_enabled("screencast.start", false);
            page.action_set_enabled("screencast.stop", true);

            let identifier = WindowIdentifier::from_native(&root).await;

            match screencast(&identifier, multiple, sources, cursor_mode).await {
                Ok((streams, fd, session)) => {
                    streams.iter().for_each(|stream| {
                        let paintable = CameraPaintable::new();
                        let picture = gtk::Picture::new();
                        picture.set_paintable(Some(&paintable));
                        picture.set_size_request(400, 400);
                        paintable.set_pipewire_node_id(fd, stream.pipe_wire_node_id());
                        self_.streams_carousel.append(&picture);
                    });

                    self_.response_group.show();
                    self_.session.lock().await.replace(session);
                    self_.close_session_btn.set_sensitive(true);
                }
                Err(err) => {
                    tracing::error!("{:#?}", err);
                    page.stop_session();
                }
            };
        }));
    }

    pub fn stop_session(&self) {
        let ctx = glib::MainContext::default();
        self.action_set_enabled("screencast.start", true);
        self.action_set_enabled("screencast.stop", false);

        ctx.spawn_local(clone!(@weak self as page => async move {
            let self_ = imp::ScreenCastPage::from_instance(&page);
            if let Some(session) = self_.session.lock().await.take() {
                let _ = session.close().await;
            }
            while let Some(child) = self_.streams_carousel.next_sibling() {
                self_.streams_carousel.remove(&child);
            }
            //paintable.close_pipeline();
            self_.response_group.hide();
            self_.close_session_btn.set_sensitive(false);
        }));
    }
}

pub async fn screencast(
    identifier: &WindowIdentifier,
    multiple: bool,
    types: BitFlags<SourceType>,
    cursor_mode: BitFlags<CursorMode>,
) -> ashpd::Result<(Vec<Stream>, RawFd, SessionProxy<'static>)> {
    let connection = zbus::azync::Connection::session().await?;
    let proxy = ScreenCastProxy::new(&connection).await?;
    let session = proxy.create_session().await?;

    proxy
        .select_sources(&session, cursor_mode, types, multiple)
        .await?;
    let streams = proxy.start(&session, identifier).await?.to_vec();

    let fd = proxy.open_pipe_wire_remote(&session).await?;
    Ok((streams, fd, session))
}

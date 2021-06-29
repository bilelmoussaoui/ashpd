use crate::widgets::CameraPaintable;
use adw::prelude::*;
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
        pub session: Arc<Mutex<Option<SessionProxy<'static>>>>,
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
        let ctx = glib::MainContext::default();
        println!("starting session");
        ctx.spawn_local(clone!(@weak self as page => async move {
        let self_ = imp::ScreenCastPage::from_instance(&page);
            let types = match self_.types_comborow.selected() {
                0 => BitFlags::<SourceType>::from_flag(SourceType::Monitor),
                1 => BitFlags::<SourceType>::from_flag(SourceType::Window),
                _ => SourceType::Monitor | SourceType::Window,
            };
            let cursor_mode = match self_.cursor_comborow.selected() {
                0 => BitFlags::<CursorMode>::from_flag(CursorMode::Hidden),
                1 => BitFlags::<CursorMode>::from_flag(CursorMode::Embedded),
                _ => BitFlags::<CursorMode>::from_flag(CursorMode::Metadata),
            };
            let multiple = self_.multiple_switch.is_active();

            let root = page.root().unwrap();
            let identifier = WindowIdentifier::from_window(&root).await;



            match screencast(identifier, multiple, types, cursor_mode).await {
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
                    println!("{:#?}", err);
                }
            };
        }));
    }

    pub fn stop_session(&self) {
        let ctx = glib::MainContext::default();
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
    window_identifier: WindowIdentifier,
    multiple: bool,
    types: BitFlags<SourceType>,
    cursor_mode: BitFlags<CursorMode>,
) -> Result<(Vec<Stream>, RawFd, SessionProxy<'static>), ashpd::Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = ScreenCastProxy::new(&connection).await?;
    let session = proxy.create_session().await?;

    proxy
        .select_sources(&session, cursor_mode, types, multiple)
        .await?;
    let streams = proxy.start(&session, window_identifier).await?.to_vec();

    let node_id = proxy.open_pipe_wire_remote(&session).await?;
    Ok((streams, node_id, session))
}

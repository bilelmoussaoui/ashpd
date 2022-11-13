use std::{os::unix::io::RawFd, sync::Arc};

use adw::prelude::*;
use ashpd::{
    desktop::{
        screencast::{CursorMode, PersistMode, Screencast, SourceType, Stream},
        Session,
    },
    enumflags2::BitFlags,
    WindowIdentifier,
};
use futures::lock::Mutex;
use gtk::{
    glib::{self, clone},
    subclass::prelude::*,
};

use crate::widgets::{
    CameraPaintable, NotificationKind, PortalPage, PortalPageExt, PortalPageImpl,
};

mod imp {
    use adw::subclass::prelude::*;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/screencast.ui")]
    pub struct ScreenCastPage {
        #[template_child]
        pub streams_carousel: TemplateChild<adw::Carousel>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub multiple_switch: TemplateChild<gtk::Switch>,
        pub session: Arc<Mutex<Option<Session<'static>>>>,
        #[template_child]
        pub monitor_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub window_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub virtual_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub cursor_mode_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub persist_mode_combo: TemplateChild<adw::ComboRow>,
        pub session_token: Arc<Mutex<Option<String>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScreenCastPage {
        const NAME: &'static str = "ScreenCastPage";
        type Type = super::ScreenCastPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async(
                "screencast.start",
                None,
                move |page, _action, _target| async move {
                    page.start_session().await;
                },
            );
            klass.install_action_async(
                "screencast.stop",
                None,
                move |page, _action, _target| async move {
                    page.stop_session().await;
                    page.send_notification("Screen cast session stopped", NotificationKind::Info);
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for ScreenCastPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().action_set_enabled("screencast.stop", false);
        }
    }
    impl WidgetImpl for ScreenCastPage {
        fn map(&self) {
            let widget = self.obj();
            let ctx = glib::MainContext::default();
            ctx.spawn_local(clone!(@weak widget as page => async move {
                let imp = page.imp();
                if let Ok((cursor_modes, source_types)) = available_types().await {
                    imp.virtual_check.set_sensitive(source_types.contains(SourceType::Virtual));
                    imp.monitor_check.set_sensitive(source_types.contains(SourceType::Monitor));
                    imp.window_check.set_sensitive(source_types.contains(SourceType::Window));
//                    imp.hidden_check.set_sensitive(cursor_modes.contains(CursorMode::Hidden));
//                    imp.metadata_check.set_sensitive(cursor_modes.contains(CursorMode::Metadata));
//                    imp.embedded_check.set_sensitive(cursor_modes.contains(CursorMode::Embedded));
                }
            }));
            self.parent_map();
        }
    }
    impl BinImpl for ScreenCastPage {}
    impl PortalPageImpl for ScreenCastPage {}
}

glib::wrapper! {
    pub struct ScreenCastPage(ObjectSubclass<imp::ScreenCastPage>)
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl ScreenCastPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    /// Returns the selected SourceType
    fn selected_sources(&self) -> BitFlags<SourceType> {
        let imp = self.imp();
        let mut sources: BitFlags<SourceType> = BitFlags::empty();
        if imp.monitor_check.is_active() {
            sources.insert(SourceType::Monitor);
        }
        if imp.window_check.is_active() {
            sources.insert(SourceType::Window);
        }
        if imp.virtual_check.is_active() {
            sources.insert(SourceType::Virtual);
        }
        sources
    }

    /// Returns the selected CursorMode
    fn selected_cursor_mode(&self) -> CursorMode {
        match self.imp().cursor_mode_combo.selected() {
            0 => CursorMode::Hidden,
            1 => CursorMode::Embedded,
            2 => CursorMode::Metadata,
            _ => unreachable!(),
        }
    }

    fn selected_persist_mode(&self) -> PersistMode {
        match self.imp().persist_mode_combo.selected() {
            0 => PersistMode::DoNot,
            1 => PersistMode::Application,
            2 => PersistMode::ExplicitlyRevoked,
            _ => unreachable!(),
        }
    }

    async fn start_session(&self) {
        let imp = self.imp();
        self.action_set_enabled("screencast.start", false);
        self.action_set_enabled("screencast.stop", true);

        match self.screencast().await {
            Ok((streams, fd, session)) => {
                self.send_notification(
                    "Screen cast session started successfully",
                    NotificationKind::Success,
                );
                streams.iter().for_each(|stream| {
                    let paintable = CameraPaintable::default();
                    let picture = gtk::Picture::builder()
                        .paintable(&paintable)
                        .hexpand(true)
                        .vexpand(true)
                        .build();
                    paintable.set_pipewire_node_id(fd, Some(stream.pipe_wire_node_id()));
                    imp.streams_carousel.append(&picture);
                });

                imp.response_group.show();
                imp.session.lock().await.replace(session);
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
        let imp = self.imp();

        self.action_set_enabled("screencast.start", true);
        self.action_set_enabled("screencast.stop", false);

        if let Some(session) = imp.session.lock().await.take() {
            let _ = session.close().await;
        }
        if let Some(mut child) = imp.streams_carousel.first_child() {
            loop {
                let picture = child.downcast_ref::<gtk::Picture>().unwrap();
                let paintable = picture
                    .paintable()
                    .unwrap()
                    .downcast::<CameraPaintable>()
                    .unwrap();
                paintable.close_pipeline();
                imp.streams_carousel.remove(picture);

                if let Some(next_child) = child.next_sibling() {
                    child = next_child;
                } else {
                    break;
                }
            }
        }

        imp.response_group.hide();
    }

    async fn screencast(&self) -> ashpd::Result<(Vec<Stream>, RawFd, Session<'static>)> {
        let imp = self.imp();
        let sources = self.selected_sources();
        let cursor_mode = self.selected_cursor_mode();
        let persist_mode = self.selected_persist_mode();
        let multiple = imp.multiple_switch.is_active();

        let root = self.native().unwrap();

        let identifier = WindowIdentifier::from_native(&root).await;

        let proxy = Screencast::new().await?;
        let session = proxy.create_session().await?;
        let mut token = imp.session_token.lock().await;
        proxy
            .select_sources(
                &session,
                cursor_mode,
                sources,
                multiple,
                token.as_deref(),
                persist_mode,
            )
            .await?;
        self.send_notification("Starting a screen cast session", NotificationKind::Info);
        let (streams, new_token) = proxy.start(&session, &identifier).await?;
        if let Some(t) = new_token {
            token.replace(t);
        }
        let fd = proxy.open_pipe_wire_remote(&session).await?;
        Ok((streams, fd, session))
    }
}

pub async fn available_types() -> ashpd::Result<(BitFlags<CursorMode>, BitFlags<SourceType>)> {
    let proxy = Screencast::new().await?;

    let cursor_modes = proxy.available_cursor_modes().await?;
    let source_types = proxy.available_source_types().await?;

    Ok((cursor_modes, source_types))
}

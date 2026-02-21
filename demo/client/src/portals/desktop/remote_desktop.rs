use std::sync::Arc;

use adw::{prelude::*, subclass::prelude::*};
use ashpd::{
    WindowIdentifier,
    desktop::{
        PersistMode, Session,
        clipboard::Clipboard,
        remote_desktop::{DeviceType, RemoteDesktop, SelectDevicesOptions},
        screencast::{CursorMode, Screencast, SelectSourcesOptions, SourceType, Stream},
    },
    enumflags2::BitFlags,
};
use futures_util::lock::Mutex;
use gtk::glib::{self, clone};

use crate::{
    portals::spawn_tokio,
    widgets::{PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
    use super::*;
    use crate::portals::desktop::screencast::available_types;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/remote_desktop.ui")]
    pub struct RemoteDesktopPage {
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        pub session: Arc<Mutex<Option<Session<RemoteDesktop>>>>,
        #[template_child]
        pub screencast_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub multiple_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub keyboard_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub pointer_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub touchscreen_check: TemplateChild<gtk::CheckButton>,
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
    impl ObjectSubclass for RemoteDesktopPage {
        const NAME: &'static str = "RemoteDesktopPage";
        type Type = super::RemoteDesktopPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async("remote_desktop.start", None, |page, _, _| async move {
                page.start_session().await;
            });
            klass.install_action_async("remote_desktop.stop", None, |page, _, _| async move {
                page.stop_session().await;
                page.info("Remote desktop session stopped");
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for RemoteDesktopPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().action_set_enabled("remote_desktop.stop", false);
        }
    }
    impl WidgetImpl for RemoteDesktopPage {
        fn map(&self) {
            let widget = self.obj();
            glib::spawn_future_local(clone!(
                #[weak]
                widget,
                async move {
                    let imp = widget.imp();
                    if let Ok((cursor_modes, source_types)) = available_types().await {
                        imp.virtual_check
                            .set_sensitive(source_types.contains(SourceType::Virtual));
                        imp.monitor_check
                            .set_sensitive(source_types.contains(SourceType::Monitor));
                        imp.window_check
                            .set_sensitive(source_types.contains(SourceType::Window));
                        let model = gtk::StringList::default();
                        if cursor_modes.contains(CursorMode::Hidden) {
                            model.append("Hidden");
                        }
                        if cursor_modes.contains(CursorMode::Metadata) {
                            model.append("Metadata");
                        }
                        if cursor_modes.contains(CursorMode::Embedded) {
                            model.append("Embedded");
                        }
                        imp.cursor_mode_combo.set_model(Some(&model));
                    }

                    if let Ok(devices) = available_devices().await {
                        imp.touchscreen_check
                            .set_sensitive(devices.contains(DeviceType::Touchscreen));
                        imp.pointer_check
                            .set_sensitive(devices.contains(DeviceType::Pointer));
                        imp.keyboard_check
                            .set_sensitive(devices.contains(DeviceType::Keyboard));
                    }
                }
            ));

            glib::spawn_future_local(glib::clone!(
                #[weak]
                widget,
                async move {
                    if let Ok(proxy) = spawn_tokio(async { RemoteDesktop::new().await }).await {
                        widget.set_property("portal-version", proxy.version());
                    }
                }
            ));
            self.parent_map();
        }
    }
    impl BinImpl for RemoteDesktopPage {}
    impl PortalPageImpl for RemoteDesktopPage {}
}

glib::wrapper! {
    pub struct RemoteDesktopPage(ObjectSubclass<imp::RemoteDesktopPage>)
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl RemoteDesktopPage {
    /// Returns the selected DeviceType
    fn selected_devices(&self) -> BitFlags<DeviceType> {
        let imp = self.imp();

        let mut devices: BitFlags<DeviceType> = BitFlags::empty();
        if imp.keyboard_check.is_active() {
            devices.insert(DeviceType::Keyboard);
        }
        if imp.pointer_check.is_active() {
            devices.insert(DeviceType::Pointer);
        }
        if imp.touchscreen_check.is_active() {
            devices.insert(DeviceType::Touchscreen);
        }
        devices
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
        sources
    }

    fn selected_persist_mode(&self) -> PersistMode {
        match self.imp().persist_mode_combo.selected() {
            0 => PersistMode::DoNot,
            1 => PersistMode::Application,
            2 => PersistMode::ExplicitlyRevoked,
            _ => unreachable!(),
        }
    }

    /// Returns the selected CursorMode
    fn selected_cursor_mode(&self) -> CursorMode {
        match self
            .imp()
            .cursor_mode_combo
            .selected_item()
            .and_downcast::<gtk::StringObject>()
            .unwrap()
            .string()
            .as_ref()
        {
            "Hidden" => CursorMode::Hidden,
            "Embedded" => CursorMode::Embedded,
            "Metadata" => CursorMode::Metadata,
            _ => unreachable!(),
        }
    }

    async fn start_session(&self) {
        let imp = self.imp();

        self.action_set_enabled("remote_desktop.start", false);
        self.action_set_enabled("remote_desktop.stop", true);

        match self.remote().await {
            Ok((_selected_devices, _streams, session, clipboard_enabled)) => {
                imp.response_group.set_visible(true);

                if let Some(old_session) = imp.session.lock().await.replace(session) {
                    spawn_tokio(async move {
                        let _ = old_session.close().await;
                    })
                    .await;
                }
                self.action_set_enabled("remote_desktop.start", false);
                self.action_set_enabled("remote_desktop.stop", true);
                self.success(&format!(
                    "Remote desktop session started successfully (with{} Clipboard)",
                    if clipboard_enabled { "" } else { "out" }
                ));
            }
            Err(err) => {
                tracing::error!("Failed to start remote desktop session: {err}");
                self.error(&format!("Failed to start a remote desktop session: {err}"));
                self.stop_session().await;
            }
        };
    }

    async fn stop_session(&self) {
        self.action_set_enabled("remote_desktop.start", true);
        self.action_set_enabled("remote_desktop.stop", false);

        let imp = self.imp();
        if let Some(session) = imp.session.lock().await.take() {
            spawn_tokio(async move {
                let _ = session.close().await;
            })
            .await;
        }
        imp.response_group.set_visible(false);
    }

    async fn remote(
        &self,
    ) -> ashpd::Result<(
        BitFlags<DeviceType>,
        Vec<Stream>,
        Session<RemoteDesktop>,
        bool,
    )> {
        let imp = self.imp();
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let is_screencast = imp.screencast_switch.get().is_active();
        let multiple_sources = imp.multiple_switch.is_active();
        let cursor_mode = self.selected_cursor_mode();
        let sources = self.selected_sources();
        let devices = self.selected_devices();
        let persist_mode = self.selected_persist_mode();
        let prev_token = imp
            .session_token
            .lock()
            .await
            .as_deref()
            .map(ToOwned::to_owned);

        self.info("Starting a remote desktop session");
        let (response_devices, response_streams, session, new_token, clipboard_enabled) =
            spawn_tokio(async move {
                let proxy = RemoteDesktop::new().await?;
                let session = proxy.create_session(Default::default()).await?;

                if proxy.version() >= 2 {
                    let clipboard = Clipboard::new().await?;
                    if let Err(e) = clipboard.request(&session, Default::default()).await {
                        tracing::error!("failed to request clipboard access: {}", e);
                    }
                }

                if is_screencast {
                    let screencast_proxy = Screencast::new().await?;
                    screencast_proxy
                        .select_sources(
                            &session,
                            SelectSourcesOptions::default()
                                .set_cursor_mode(cursor_mode)
                                .set_sources(sources)
                                .set_multiple(multiple_sources)
                                .set_restore_token(None)
                                .set_persist_mode(PersistMode::DoNot),
                        )
                        .await?;
                }
                proxy
                    .select_devices(
                        &session,
                        SelectDevicesOptions::default()
                            .set_devices(devices)
                            .set_restore_token(prev_token.as_deref())
                            .set_persist_mode(persist_mode),
                    )
                    .await?;

                let response = proxy
                    .start(&session, identifier.as_ref(), Default::default())
                    .await?
                    .response()?;

                ashpd::Result::Ok((
                    response.devices(),
                    response.streams().to_owned(),
                    session,
                    response.restore_token().map(ToOwned::to_owned),
                    response.clipboard_enabled(),
                ))
            })
            .await?;

        if let Some(t) = new_token {
            imp.session_token.lock().await.replace(t.to_owned());
        }

        Ok((
            response_devices,
            response_streams,
            session,
            clipboard_enabled,
        ))
    }
}

pub async fn available_devices() -> ashpd::Result<BitFlags<DeviceType>> {
    spawn_tokio(async {
        let proxy = RemoteDesktop::new().await?;
        proxy.available_device_types().await
    })
    .await
}

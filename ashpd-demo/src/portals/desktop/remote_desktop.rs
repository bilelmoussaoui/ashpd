use crate::widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl};
use ashpd::{
    desktop::{
        remote_desktop::{DeviceType, RemoteDesktopProxy},
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
use std::sync::Arc;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use crate::portals::desktop::screencast::available_types;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/remote_desktop.ui")]
    pub struct RemoteDesktopPage {
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        pub session: Arc<Mutex<Option<SessionProxy<'static>>>>,
        #[template_child]
        pub screencast_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub multiple_switch: TemplateChild<gtk::Switch>,
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
        pub hidden_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub embedded_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub metadata_check: TemplateChild<gtk::CheckButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RemoteDesktopPage {
        const NAME: &'static str = "RemoteDesktopPage";
        type Type = super::RemoteDesktopPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "remote_desktop.start",
                None,
                move |page, _action, _target| {
                    let ctx = glib::MainContext::default();
                    ctx.spawn_local(clone!(@weak page => async move {
                        page.start_session().await;
                    }));
                },
            );
            klass.install_action(
                "remote_desktop.stop",
                None,
                move |page, _action, _target| {
                    let ctx = glib::MainContext::default();
                    ctx.spawn_local(clone!(@weak page => async move {
                        page.stop_session().await;
                        page.send_notification("Remote desktop session stopped", NotificationKind::Info);
                    }));
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for RemoteDesktopPage {
        fn constructed(&self, obj: &Self::Type) {
            obj.action_set_enabled("remote_desktop.stop", false);
            self.parent_constructed(obj);
        }
    }
    impl WidgetImpl for RemoteDesktopPage {
        fn map(&self, widget: &Self::Type) {
            let ctx = glib::MainContext::default();
            ctx.spawn_local(clone!(@weak widget as page => async move {
                let self_ = imp::RemoteDesktopPage::from_instance(&page);
                if let Ok((cursor_modes, source_types)) = available_types().await {
                    self_.virtual_check.set_sensitive(source_types.contains(SourceType::Virtual));
                    self_.monitor_check.set_sensitive(source_types.contains(SourceType::Monitor));
                    self_.window_check.set_sensitive(source_types.contains(SourceType::Window));
                    self_.hidden_check.set_sensitive(cursor_modes.contains(CursorMode::Hidden));
                    self_.metadata_check.set_sensitive(cursor_modes.contains(CursorMode::Metadata));
                    self_.embedded_check.set_sensitive(cursor_modes.contains(CursorMode::Embedded));
                }

                if let Ok(devices) = available_devices().await {
                    self_.touchscreen_check.set_sensitive(devices.contains(DeviceType::Touchscreen));
                    self_.pointer_check.set_sensitive(devices.contains(DeviceType::Pointer));
                    self_.keyboard_check.set_sensitive(devices.contains(DeviceType::Keyboard));
                }
            }));
            self.parent_map(widget);
        }
    }
    impl BinImpl for RemoteDesktopPage {}
    impl PortalPageImpl for RemoteDesktopPage {}
}

glib::wrapper! {
    pub struct RemoteDesktopPage(ObjectSubclass<imp::RemoteDesktopPage>) @extends gtk::Widget, adw::Bin, PortalPage;
}

impl RemoteDesktopPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a RemoteDesktopPage")
    }

    /// Returns the selected DeviceType
    fn selected_devices(&self) -> BitFlags<DeviceType> {
        let self_ = imp::RemoteDesktopPage::from_instance(self);

        let mut devices: BitFlags<DeviceType> = BitFlags::empty();
        if self_.keyboard_check.is_active() {
            devices.insert(DeviceType::Keyboard);
        }
        if self_.pointer_check.is_active() {
            devices.insert(DeviceType::Pointer);
        }
        if self_.touchscreen_check.is_active() {
            devices.insert(DeviceType::Touchscreen);
        }
        devices
    }

    /// Returns the selected SourceType
    fn selected_sources(&self) -> BitFlags<SourceType> {
        let self_ = imp::RemoteDesktopPage::from_instance(self);
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
        let self_ = imp::RemoteDesktopPage::from_instance(self);

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
        let self_ = imp::RemoteDesktopPage::from_instance(self);

        self.action_set_enabled("remote_desktop.start", false);
        self.action_set_enabled("remote_desktop.stop", true);

        match self.remote().await {
            Ok((_selected_devices, _streams, session)) => {
                self_.response_group.show();
                self_.session.lock().await.replace(session);
                self.action_set_enabled("remote_desktop.start", false);
                self.action_set_enabled("remote_desktop.stop", true);
                self.send_notification(
                    "Remote desktop session started successfully",
                    NotificationKind::Success,
                );
            }
            Err(err) => {
                tracing::error!("{:#?}", err);
                self.send_notification(
                    "Failed to start a remote desktop session",
                    NotificationKind::Error,
                );
                self.stop_session().await;
            }
        };
    }

    async fn stop_session(&self) {
        self.action_set_enabled("remote_desktop.start", true);
        self.action_set_enabled("remote_desktop.stop", false);

        let self_ = imp::RemoteDesktopPage::from_instance(self);
        if let Some(session) = self_.session.lock().await.take() {
            let _ = session.close().await;
        }
        self_.response_group.hide();
    }

    async fn remote(
        &self,
    ) -> ashpd::Result<(BitFlags<DeviceType>, Vec<Stream>, SessionProxy<'static>)> {
        let self_ = imp::RemoteDesktopPage::from_instance(self);
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let is_screencast = self_.screencast_switch.get().is_active();
        let multiple_sources = self_.multiple_switch.is_active();
        let cursor_mode = self.selected_cursor_mode();
        let sources = self.selected_sources();
        let devices = self.selected_devices();

        let connection = zbus::Connection::session().await?;
        let proxy = RemoteDesktopProxy::new(&connection).await?;
        let session = proxy.create_session().await?;
        if is_screencast {
            let screencast_proxy = ScreenCastProxy::new(&connection).await?;
            screencast_proxy
                .select_sources(&session, cursor_mode, sources, multiple_sources)
                .await?;
        }
        proxy.select_devices(&session, devices).await?;

        self.send_notification("Starting a remote desktop session", NotificationKind::Info);
        let (devices, streams) = proxy.start(&session, &identifier).await?;
        Ok((devices, streams, session))
    }
}

pub async fn available_devices() -> ashpd::Result<BitFlags<DeviceType>> {
    let cnx = zbus::Connection::session().await?;
    let proxy = RemoteDesktopProxy::new(&cnx).await?;
    proxy.available_device_types().await
}

use std::sync::Arc;

use adw::prelude::*;
use ashpd::{
    desktop::{
        remote_desktop::{DeviceType, KeyState, RemoteDesktopProxy},
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

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/remote_desktop.ui")]
    pub struct RemoteDesktopPage {
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        pub session: Arc<Mutex<Option<SessionProxy<'static>>>>,
        #[template_child]
        pub close_session_btn: TemplateChild<gtk::Button>,
    }

    impl Default for RemoteDesktopPage {
        fn default() -> Self {
            Self {
                response_group: TemplateChild::default(),
                close_session_btn: TemplateChild::default(),
                session: Arc::new(Mutex::new(None)),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RemoteDesktopPage {
        const NAME: &'static str = "RemoteDesktopPage";
        type Type = super::RemoteDesktopPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action(
                "remote_desktop.start",
                None,
                move |page, _action, _target| {
                    page.start_session();
                },
            );
            klass.install_action(
                "remote_desktop.stop",
                None,
                move |page, _action, _target| {
                    page.stop_session();
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

            self.close_session_btn.set_sensitive(false);
            self.parent_constructed(obj);
        }
    }
    impl WidgetImpl for RemoteDesktopPage {}
    impl BinImpl for RemoteDesktopPage {}
}

glib::wrapper! {
    pub struct RemoteDesktopPage(ObjectSubclass<imp::RemoteDesktopPage>) @extends gtk::Widget, adw::Bin;
}

impl RemoteDesktopPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a RemoteDesktopPage")
    }

    pub fn start_session(&self) {
        let ctx = glib::MainContext::default();
        ctx.spawn_local(clone!(@weak self as page => async move {
            let self_ = imp::RemoteDesktopPage::from_instance(&page);

            let root = page.native().unwrap();
            page.action_set_enabled("remote_desktop.start", false);
            page.action_set_enabled("remote_desktop.stop", true);

            let identifier = WindowIdentifier::from_native(&root).await;

            match remote(&identifier,DeviceType::Keyboard | DeviceType::Pointer).await {
                Ok((devices, streams, session)) => {
                    self_.response_group.show();
                    self_.session.lock().await.replace(session);
                    self_.close_session_btn.set_sensitive(true);
                }
                Err(err) => {
                    tracing::error!("{:#?}", err);
                }
            };
        }));
    }

    pub fn stop_session(&self) {
        let ctx = glib::MainContext::default();
        self.action_set_enabled("remote_desktop.start", true);
        self.action_set_enabled("remote_desktop.stop", false);

        ctx.spawn_local(clone!(@weak self as page => async move {
            let self_ = imp::RemoteDesktopPage::from_instance(&page);
            if let Some(session) = self_.session.lock().await.take() {
                let _ = session.close().await;
            }
            self_.response_group.hide();
            self_.close_session_btn.set_sensitive(false);
        }));
    }
}

pub async fn remote(
    identifier: &WindowIdentifier,
    devices: BitFlags<DeviceType>,
) -> ashpd::Result<(BitFlags<DeviceType>, Vec<Stream>, SessionProxy<'static>)> {
    let connection = zbus::azync::Connection::session().await?;
    let proxy = RemoteDesktopProxy::new(&connection).await?;
    let screencast = ScreenCastProxy::new(&connection).await?;

    let session = proxy.create_session().await?;
    proxy.select_devices(&session, devices).await?;

    screencast
        .select_sources(
            &session,
            CursorMode::Metadata.into(),
            SourceType::Monitor | SourceType::Window,
            true,
        )
        .await?;

    let (devices, streams) = proxy.start(&session, identifier).await?;
    Ok((devices, streams, session))
}

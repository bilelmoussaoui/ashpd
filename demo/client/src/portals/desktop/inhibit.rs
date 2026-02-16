use std::sync::Arc;

use adw::subclass::prelude::*;
use ashpd::{
    WindowIdentifier,
    desktop::{
        Session,
        inhibit::{InhibitFlags, InhibitProxy, SessionState},
    },
    enumflags2::BitFlags,
};
use futures_util::lock::Mutex;
use gtk::{glib, prelude::*};

use crate::{
    portals::spawn_tokio,
    widgets::{PortalPage, PortalPageImpl},
};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/inhibit.ui")]
    pub struct InhibitPage {
        #[template_child]
        pub reason: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub idle_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub logout_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub user_switch_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub suspend_check: TemplateChild<gtk::CheckButton>,
        pub session: Arc<Mutex<Option<Session<InhibitProxy>>>>,
        pub task_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InhibitPage {
        const NAME: &'static str = "InhibitPage";
        type Type = super::InhibitPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async("inhibit.start_session", None, |page, _, _| async move {
                if let Err(err) = page.start_session().await {
                    tracing::error!("Failed to inhibit {}", err);
                }
            });
            klass.install_action_async("inhibit.stop", None, |page, _, _| async move {
                page.stop().await;
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for InhibitPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().action_set_enabled("inhibit.stop", false);
        }
    }
    impl WidgetImpl for InhibitPage {
        fn map(&self) {
            self.parent_map();
            let obj = self.obj();

            glib::spawn_future_local(glib::clone!(
                #[weak]
                obj,
                async move {
                    if let Ok(proxy) = spawn_tokio(async { InhibitProxy::new().await }).await {
                        obj.set_property("portal-version", proxy.version());
                    }
                }
            ));
        }
    }
    impl BinImpl for InhibitPage {}
    impl PortalPageImpl for InhibitPage {}
}

glib::wrapper! {
    pub struct InhibitPage(ObjectSubclass<imp::InhibitPage>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl InhibitPage {
    fn inhibit_flags(&self) -> BitFlags<InhibitFlags> {
        let imp = self.imp();
        let mut flags = BitFlags::empty();

        if imp.user_switch_check.is_active() {
            flags.insert(InhibitFlags::UserSwitch);
        }
        if imp.suspend_check.is_active() {
            flags.insert(InhibitFlags::Suspend);
        }
        if imp.idle_check.is_active() {
            flags.insert(InhibitFlags::Idle);
        }
        if imp.logout_check.is_active() {
            flags.insert(InhibitFlags::Logout);
        }

        flags
    }

    async fn start_session(&self) -> ashpd::Result<()> {
        let root = self.native().unwrap();
        let imp = self.imp();
        let identifier = WindowIdentifier::from_native(&root).await;

        let monitor = spawn_tokio(async move {
            let proxy = InhibitProxy::new().await?;
            let monitor = proxy.create_monitor(identifier.as_ref()).await?;
            ashpd::Result::Ok(monitor)
        })
        .await?;

        if let Some(old_session) = imp.session.lock().await.replace(monitor) {
            spawn_tokio(async move {
                let _ = old_session.close().await;
            })
            .await;
        }
        self.action_set_enabled("inhibit.stop", true);
        self.action_set_enabled("inhibit.start_session", false);

        let (sender, receiver_glib) = async_channel::unbounded();

        let page = self.clone();
        glib::spawn_future_local(async move {
            while let Ok(state_event) = receiver_glib.recv().await {
                page.on_state_changed(state_event).await;
            }
        });

        let task_handle = crate::portals::RUNTIME.spawn(async move {
            use futures_util::StreamExt;

            let proxy = match InhibitProxy::new().await {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("Failed to create InhibitProxy: {e}");
                    return;
                }
            };

            let mut stream = match proxy.receive_state_changed().await {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Failed to receive state changed: {e}");
                    return;
                }
            };

            while let Some(state_event) = stream.next().await {
                if sender.send(state_event).await.is_err() {
                    break;
                }
            }
        });

        imp.task_handle.lock().await.replace(task_handle);

        Ok(())
    }

    async fn stop(&self) {
        let imp = self.imp();
        self.action_set_enabled("inhibit.stop", false);
        self.action_set_enabled("inhibit.start_session", true);
        if let Some(handle) = imp.task_handle.lock().await.take() {
            handle.abort();
        }
        if let Some(session) = imp.session.lock().await.take() {
            spawn_tokio(async move {
                let _ = session.close().await;
            })
            .await;
        }
    }

    async fn on_state_changed(&self, state_event: ashpd::desktop::inhibit::InhibitState) {
        let imp = self.imp();
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let reason = imp.reason.text().to_string();
        let flags = self.inhibit_flags();

        match state_event.session_state() {
            SessionState::Running => {
                tracing::info!("Session running");
            }
            SessionState::QueryEnd => {
                tracing::info!("Session: query end");

                let session = imp.session.clone();
                let result = spawn_tokio(async move {
                    let proxy = InhibitProxy::new().await?;
                    proxy.inhibit(identifier.as_ref(), flags, &reason).await?;

                    if let Some(session) = session.lock().await.as_ref() {
                        proxy.query_end_response(session).await?;
                    }
                    ashpd::Result::Ok(())
                })
                .await;

                if let Err(err) = result {
                    tracing::error!("Failed to handle query end: {err}");
                }
            }
            SessionState::Ending => {
                tracing::info!("Ending the session");
            }
        }
    }
}

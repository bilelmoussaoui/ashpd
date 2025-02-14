use std::sync::Arc;

use adw::subclass::prelude::*;
use ashpd::{
    desktop::{
        inhibit::{InhibitFlags, InhibitProxy, SessionState},
        Session,
    },
    enumflags2::BitFlags,
    WindowIdentifier,
};
use futures_util::{lock::Mutex, stream::StreamExt};
use gtk::{glib, prelude::*};

use crate::widgets::{PortalPage, PortalPageImpl};

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
        pub session: Arc<Mutex<Option<Session<'static, InhibitProxy<'static>>>>>,
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
    impl WidgetImpl for InhibitPage {}
    impl BinImpl for InhibitPage {}
    impl PortalPageImpl for InhibitPage {}
}

glib::wrapper! {
    pub struct InhibitPage(ObjectSubclass<imp::InhibitPage>)
        @extends gtk::Widget, adw::Bin;
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
        let reason = imp.reason.text();
        let flags = self.inhibit_flags();

        let proxy = InhibitProxy::new().await?;
        let monitor = proxy.create_monitor(identifier.as_ref()).await?;

        imp.session.lock().await.replace(monitor);
        self.action_set_enabled("inhibit.stop", true);
        self.action_set_enabled("inhibit.start_session", false);

        let mut state = proxy.receive_state_changed().await?;
        match state
            .next()
            .await
            .expect("Stream exhausted")
            .session_state()
        {
            SessionState::Running => tracing::info!("Session running"),
            SessionState::QueryEnd => {
                tracing::info!("Session: query end");
                proxy.inhibit(identifier.as_ref(), flags, &reason).await?;
                if let Some(session) = imp.session.lock().await.as_ref() {
                    proxy.query_end_response(session).await?;
                }
            }
            SessionState::Ending => {
                tracing::info!("Ending the session");
            }
        }
        Ok(())
    }

    async fn stop(&self) {
        let imp = self.imp();
        self.action_set_enabled("inhibit.stop", false);
        self.action_set_enabled("inhibit.start_session", true);
        if let Some(session) = imp.session.lock().await.take() {
            let _ = session.close().await;
        }
    }
}

use ashpd::{desktop::inhibit, zbus, WindowIdentifier};
use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk_macros::spawn;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/inhibit.ui")]
    pub struct InhibitPage {
        #[template_child]
        pub reason: TemplateChild<gtk::Entry>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InhibitPage {
        const NAME: &'static str = "InhibitPage";
        type Type = super::InhibitPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("inhibit.request", None, move |page, _action, _target| {
                page.inhibit();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for InhibitPage {}
    impl WidgetImpl for InhibitPage {}
    impl BinImpl for InhibitPage {}
}

glib::wrapper! {
    pub struct InhibitPage(ObjectSubclass<imp::InhibitPage>) @extends gtk::Widget, adw::Bin;
}

impl InhibitPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a InhibitPage")
    }

    fn inhibit(&self) {
        let root = self.native().unwrap();
        spawn!(clone!(@weak self as page => async move {
            let self_ = imp::InhibitPage::from_instance(&page);
            let identifier = WindowIdentifier::from_native(&root).await;
            let reason = self_.reason.text();

            if let Err(err) = inhibit(&identifier, &reason).await
            {
                tracing::error!("Failed to request to inhibit stuff {}", err);
            }
        }));
    }
}

async fn inhibit(identifier: &WindowIdentifier, reason: &str) -> ashpd::Result<()> {
    let connection = zbus::azync::Connection::session().await?;
    let proxy = inhibit::InhibitProxy::new(&connection).await?;
    let monitor = proxy.create_monitor(&identifier).await?;
    let state = proxy.receive_state_changed().await?;
    match state.session_state() {
        inhibit::SessionState::Running => (),
        inhibit::SessionState::QueryEnd => {
            proxy
                .inhibit(
                    &identifier,
                    inhibit::InhibitFlags::Logout | inhibit::InhibitFlags::UserSwitch,
                    reason,
                )
                .await?;
            proxy.query_end_response(&monitor).await?;
        }
        inhibit::SessionState::Ending => {
            tracing::info!("Ending the session");
        }
    }
    Ok(())
}

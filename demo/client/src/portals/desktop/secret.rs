use std::sync::{Arc, Mutex};

use adw::subclass::prelude::*;
use ashpd::desktop::secret;
use gtk::{glib, prelude::*};

use crate::{
    portals::spawn_tokio,
    widgets::{PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/secret.ui")]
    pub struct SecretPage {
        #[template_child]
        pub token_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        pub key: Arc<Mutex<Option<String>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SecretPage {
        const NAME: &'static str = "SecretPage";
        type Type = super::SecretPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async("secret.retrieve", None, |page, _, _| async move {
                page.retrieve_secret().await;
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for SecretPage {}
    impl WidgetImpl for SecretPage {}
    impl BinImpl for SecretPage {}
    impl PortalPageImpl for SecretPage {}
}

glib::wrapper! {
    pub struct SecretPage(ObjectSubclass<imp::SecretPage>)
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl SecretPage {
    async fn retrieve_secret(&self) {
        let imp = self.imp();

        match spawn_tokio(async { secret::retrieve().await }).await {
            Ok(key) => {
                let key_str = format!("{key:?}")
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .replace(',', " ");
                imp.token_label.set_text(&key_str);
                imp.response_group.set_visible(true);
            }
            Err(err) => {
                tracing::error!("Failed to retrieve secret: {err}");
                self.error("Failed to retrieve secret");
            }
        }
    }
}

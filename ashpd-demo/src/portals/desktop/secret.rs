use std::sync::{Arc, Mutex};

use ashpd::{desktop::secret, zbus};
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/secret.ui")]
    pub struct SecretPage {
        #[template_child]
        pub token_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        pub key: Arc<Mutex<Option<String>>>,
    }

    impl Default for SecretPage {
        fn default() -> Self {
            Self {
                key: Arc::new(Mutex::new(None)),
                token_label: TemplateChild::default(),
                response_group: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SecretPage {
        const NAME: &'static str = "SecretPage";
        type Type = super::SecretPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("secret.retrieve", None, move |page, _action, _target| {
                page.retrieve_secret();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for SecretPage {}
    impl WidgetImpl for SecretPage {}
    impl BinImpl for SecretPage {}
}

glib::wrapper! {
    pub struct SecretPage(ObjectSubclass<imp::SecretPage>) @extends gtk::Widget, adw::Bin;
}

impl SecretPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a SecretPage")
    }

    pub fn retrieve_secret(&self) {
        let ctx = glib::MainContext::default();
        ctx.spawn_local(clone!(@weak self as page => async move {
            let self_ = imp::SecretPage::from_instance(&page);

            if let Ok(token) = retrieve_secret(None).await
            {
                tracing::debug!("Received token: {:#?}", token);
                self_.token_label.set_text(&token);
                self_.response_group.show();
            }
        }));
    }
}

async fn retrieve_secret(old_token: Option<&str>) -> ashpd::Result<String> {
    let connection = zbus::azync::Connection::session().await?;
    let proxy = secret::SecretProxy::new(&connection).await?;
    let tmp_file = glib::mkstemp("some_stuff_XXXXXX");
    let new_token = proxy.retrieve_secret(&tmp_file, old_token).await?;
    tracing::info!("Received secret {}", new_token);
    Ok(new_token)
}

use crate::widgets::{PortalPage, PortalPageImpl};
use ashpd::{desktop::secret, zbus};
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::sync::{Arc, Mutex};
use std::{fs::File, io::Read};

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
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
            Self::bind_template(klass);

            klass.install_action("secret.retrieve", None, move |page, _action, _target| {
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak page => async move {
                    page.retrieve_secret().await;
                }));
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
    pub struct SecretPage(ObjectSubclass<imp::SecretPage>) @extends gtk::Widget, adw::Bin, PortalPage;
}

impl SecretPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a SecretPage")
    }

    async fn retrieve_secret(&self) {
        let self_ = imp::SecretPage::from_instance(self);

        if let Ok(token) = retrieve_secret(None).await {
            tracing::debug!("Received token: {:#?}", token);
            self_.token_label.set_text(&token);
            self_.response_group.show();
        }
    }
}

async fn retrieve_secret(old_token: Option<&str>) -> ashpd::Result<String> {
    use glib::translate::*;
    let connection = zbus::azync::Connection::session().await?;
    let proxy = secret::SecretProxy::new(&connection).await?;

    let path: std::path::PathBuf =
        unsafe { from_glib_none(glib::ffi::g_mkdtemp("some_stuff_XXXXXX".to_glib_none().0)) };
    let mut file = File::open(path).unwrap();

    let new_token = proxy.retrieve_secret(&file, old_token).await?;
    tracing::info!("Received secret {}", new_token);

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    println!("{}", contents);
    Ok(new_token)
}

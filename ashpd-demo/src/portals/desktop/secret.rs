use std::{
    io::Read,
    sync::{Arc, Mutex},
};

use ashpd::desktop::secret;
use glib::clone;
use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::widgets::{PortalPage, PortalPageImpl};

mod imp {
    use adw::subclass::prelude::*;

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
        let imp = self.imp();

        if let Ok(key) = retrieve_secret().await {
            let key_str = format!("{:?}", key)
                .trim_start_matches('[')
                .trim_end_matches(']')
                .replace(',', " ");
            imp.token_label.set_text(&key_str);
            imp.response_group.show();
        }
    }
}

async fn retrieve_secret() -> ashpd::Result<Vec<u8>> {
    let proxy = secret::Secret::new().await?;

    let (mut x1, x2) = std::os::unix::net::UnixStream::pair().unwrap();
    proxy.retrieve(&x2).await?;
    drop(x2);
    let mut buf = Vec::new();
    x1.read_to_end(&mut buf)?;

    Ok(buf)
}

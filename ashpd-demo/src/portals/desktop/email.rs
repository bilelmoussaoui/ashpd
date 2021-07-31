use ashpd::{
    desktop::email::{self, Email},
    WindowIdentifier,
};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::portals::{is_empty, split_comma};

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/email.ui")]
    pub struct EmailPage {
        #[template_child]
        pub subject: TemplateChild<gtk::Entry>,
        #[template_child]
        pub body: TemplateChild<gtk::Entry>,
        #[template_child]
        pub addresses: TemplateChild<gtk::Entry>,
        #[template_child]
        pub cc_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub bcc_entry: TemplateChild<gtk::Entry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EmailPage {
        const NAME: &'static str = "EmailPage";
        type Type = super::EmailPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<adw::ClampLayout>();
            klass.install_action("email.compose", None, move |page, _action, _target| {
                page.compose_mail();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for EmailPage {}
    impl WidgetImpl for EmailPage {}
    impl BinImpl for EmailPage {}
}

glib::wrapper! {
    pub struct EmailPage(ObjectSubclass<imp::EmailPage>) @extends gtk::Widget, adw::Bin;
}

impl EmailPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a EmailPage")
    }

    pub fn compose_mail(&self) {
        let self_ = imp::EmailPage::from_instance(self);
        let subject = is_empty(self_.subject.text());
        let body = is_empty(self_.body.text());
        let addresses = is_empty(self_.addresses.text()).map(|s| split_comma(s));
        let bcc = is_empty(self_.bcc_entry.text()).map(|s| split_comma(s));
        let cc = is_empty(self_.cc_entry.text()).map(|s| split_comma(s));
        let root = self.native().unwrap();
        let ctx = glib::MainContext::default();

        let mut email = Email::new();
        if let Some(subject) = subject {
            email.set_subject(&subject);
        }
        if let Some(body) = body {
            email.set_body(&body);
        }
        if let Some(addresses) = addresses {
            email.set_addresses(&addresses);
        }
        if let Some(bcc) = bcc {
            email.set_bcc(&bcc);
        }
        if let Some(cc) = cc {
            email.set_cc(&cc);
        }

        ctx.spawn_local(async move {
            let identifier = WindowIdentifier::from_native(&root).await;
            let _ = email::compose(&identifier, email).await;
        });
    }
}

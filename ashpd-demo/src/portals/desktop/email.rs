use ashpd::{desktop::email::EmailRequest, WindowIdentifier};
use gtk::{
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

use crate::{
    portals::{is_empty, split_comma},
    widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
    use adw::subclass::prelude::*;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/email.ui")]
    pub struct EmailPage {
        #[template_child]
        pub subject: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub body: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub addresses: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub cc_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub bcc_entry: TemplateChild<adw::EntryRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EmailPage {
        const NAME: &'static str = "EmailPage";
        type Type = super::EmailPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("email.compose", None, move |page, _action, _target| {
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak page => async move {
                    page.compose_mail().await;
                }));
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for EmailPage {}
    impl WidgetImpl for EmailPage {}
    impl BinImpl for EmailPage {}
    impl PortalPageImpl for EmailPage {}
}

glib::wrapper! {
    pub struct EmailPage(ObjectSubclass<imp::EmailPage>) @extends gtk::Widget, adw::Bin, PortalPage;
}

impl EmailPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a EmailPage")
    }

    async fn compose_mail(&self) {
        let imp = self.imp();

        let subject = is_empty(imp.subject.text());
        let body = is_empty(imp.body.text());
        let addresses = is_empty(imp.addresses.text()).map(split_comma);
        let bcc = is_empty(imp.bcc_entry.text()).map(split_comma);
        let cc = is_empty(imp.cc_entry.text()).map(split_comma);
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;

        let mut request = EmailRequest::default().identifier(identifier);
        if let Some(subject) = subject {
            request.set_subject(&subject);
        }
        if let Some(addresses) = addresses {
            request.set_addresses(&addresses);
        }
        if let Some(cc) = cc {
            request.set_cc(&cc);
        }
        if let Some(bcc) = bcc {
            request.set_bcc(&bcc);
        }
        if let Some(body) = body {
            request.set_body(&body);
        }
        match request.build().await {
            Ok(_) => {
                self.send_notification(
                    "Compose an email request was successful",
                    NotificationKind::Success,
                );
            }
            Err(_err) => self.send_notification(
                "Request to compose an email failed",
                NotificationKind::Error,
            ),
        }
    }
}

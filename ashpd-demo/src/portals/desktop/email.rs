use ashpd::desktop::email::{Email, EmailProxy};
use ashpd::{zbus, WindowIdentifier};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

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
        let subject = self_.subject.text();
        let body = self_.body.text();
        let addresses = self_.addresses.text();
        let bcc = self_.bcc_entry.text();
        let cc = self_.cc_entry.text();
        let root = self.native().unwrap();
        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move {
            let identifier = WindowIdentifier::from_native(&root).await;
            let _ = compose_email(
                identifier,
                Email::default()
                    .subject(&subject)
                    .addresses(
                        &addresses
                            .split(',')
                            .filter(|e| e.len() > 1)
                            .collect::<Vec<_>>(),
                    )
                    .bcc(&bcc.split(',').filter(|e| e.len() > 1).collect::<Vec<_>>())
                    .cc(&cc.split(',').filter(|e| e.len() > 1).collect::<Vec<_>>())
                    .body(&body),
            )
            .await;
        });
    }
}

pub async fn compose_email(
    identifier: WindowIdentifier,
    email: Email,
) -> Result<(), ashpd::Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = EmailProxy::new(&connection).await?;
    proxy.compose_email(identifier, email).await?;

    Ok(())
}

use ashpd::desktop::notification::{Notification, NotificationProxy};
use ashpd::zbus;
use ashpd::{RequestProxy, Response, WindowIdentifier};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/notification.ui")]
    pub struct NotificationPage {
        #[template_child]
        pub id_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub title_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub body_entry: TemplateChild<gtk::Entry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NotificationPage {
        const NAME: &'static str = "NotificationPage";
        type Type = super::NotificationPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("notification.send", None, move |page, _action, _target| {
                page.send_notification().unwrap();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for NotificationPage {
        fn constructed(&self, obj: &Self::Type) {}
    }
    impl WidgetImpl for NotificationPage {}
    impl BinImpl for NotificationPage {}
}

glib::wrapper! {
    pub struct NotificationPage(ObjectSubclass<imp::NotificationPage>) @extends gtk::Widget, adw::Bin;
}

impl NotificationPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a NotificationPage")
    }

    pub fn send_notification(&self) -> zbus::fdo::Result<()> {
        let self_ = imp::NotificationPage::from_instance(self);

        let notification_id = self_.id_entry.get_text();
        let title = self_.title_entry.get_text();
        let body = self_.body_entry.get_text();

        let connection = zbus::Connection::new_session()?;
        let proxy = NotificationProxy::new(&connection)?;
        proxy.add_notification(&notification_id, Notification::new(&title).body(&body))?;

        Ok(())
    }
}

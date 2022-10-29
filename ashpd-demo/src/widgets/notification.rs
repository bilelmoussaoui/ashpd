use gtk::{
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

pub enum NotificationKind {
    Info,
    Success,
    Error,
}

mod imp {
    use adw::subclass::prelude::*;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/notification_widget.ui")]
    pub struct Notification {
        #[template_child]
        pub info_bar: TemplateChild<gtk::InfoBar>,
        #[template_child]
        pub message_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Notification {
        const NAME: &'static str = "Notification";
        type Type = super::Notification;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_css_name("notification");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for Notification {}
    impl WidgetImpl for Notification {}
    impl BinImpl for Notification {}
}

glib::wrapper! {
    pub struct Notification(ObjectSubclass<imp::Notification>)
        @extends gtk::Widget, adw::Bin;
}

impl Notification {
    pub fn send(&self, text: &str, kind: NotificationKind) {
        let imp = self.imp();
        imp.info_bar.remove_css_class("error");
        imp.info_bar.remove_css_class("info");
        imp.info_bar.remove_css_class("success");

        match kind {
            NotificationKind::Error => {
                imp.info_bar.set_message_type(gtk::MessageType::Error);
                imp.info_bar.add_css_class("error");
            }
            NotificationKind::Info => {
                imp.info_bar.set_message_type(gtk::MessageType::Info);
                imp.info_bar.add_css_class("info");
            }
            NotificationKind::Success => {
                imp.info_bar.set_message_type(gtk::MessageType::Other);
                imp.info_bar.add_css_class("success");
            }
        }
        imp.info_bar.set_revealed(true);
        imp.message_label.set_label(text);

        glib::timeout_add_seconds_local_once(
            3,
            clone!(@weak self as widget => move || {
                widget.close();
            }),
        );
    }

    pub fn close(&self) {
        self.imp().info_bar.set_revealed(false);
    }
}

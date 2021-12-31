use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

pub enum NotificationKind {
    Info,
    Success,
    Error,
}

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
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
            Self::bind_template(klass);
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
    pub struct Notification(ObjectSubclass<imp::Notification>) @extends gtk::Widget, adw::Bin;
}

impl Notification {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a Notification")
    }

    pub fn send(&self, text: &str, kind: NotificationKind) {
        let self_ = imp::Notification::from_instance(self);
        self_.info_bar.remove_css_class("error");
        self_.info_bar.remove_css_class("info");
        self_.info_bar.remove_css_class("success");

        match kind {
            NotificationKind::Error => {
                self_.info_bar.set_message_type(gtk::MessageType::Error);
                self_.info_bar.add_css_class("error");
            }
            NotificationKind::Info => {
                self_.info_bar.set_message_type(gtk::MessageType::Info);
                self_.info_bar.add_css_class("info");
            }
            NotificationKind::Success => {
                self_.info_bar.set_message_type(gtk::MessageType::Other);
                self_.info_bar.add_css_class("success");
            }
        }
        self_.info_bar.set_revealed(true);
        self_.message_label.set_label(text);

        glib::timeout_add_seconds_local_once(
            3,
            clone!(@weak self as widget => move || {
                widget.close();
            }),
        );
    }

    pub fn close(&self) {
        let self_ = imp::Notification::from_instance(self);
        self_.info_bar.set_revealed(false);
    }
}

use adw::{prelude::*, subclass::prelude::*};
use gtk::glib::{self, clone};

pub enum NotificationKind {
    Info,
    Success,
    Error,
}

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default)]
    pub struct Notification {
        pub banner: adw::Banner,
        pub(super) source_id: RefCell<Option<glib::SourceId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Notification {
        const NAME: &'static str = "Notification";
        type Type = super::Notification;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("notification");
        }
    }
    impl ObjectImpl for Notification {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().set_child(Some(&self.banner));
        }
    }
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
        imp.banner.remove_css_class("error");
        imp.banner.remove_css_class("info");
        imp.banner.remove_css_class("success");

        match kind {
            NotificationKind::Error => {
                imp.banner.add_css_class("error");
            }
            NotificationKind::Info => {
                imp.banner.add_css_class("info");
            }
            NotificationKind::Success => {
                imp.banner.add_css_class("success");
            }
        }
        imp.banner.set_revealed(true);
        imp.banner.set_title(text);

        if let Some(source_id) = imp.source_id.take() {
            source_id.remove();
        }

        let source_id = glib::timeout_add_seconds_local_once(
            3,
            clone!(@weak self as widget => move || {
                widget.close();
            }),
        );
        imp.source_id.replace(Some(source_id));
    }

    pub fn close(&self) {
        let imp = self.imp();
        imp.banner.set_revealed(false);
        if let Some(source_id) = imp.source_id.take() {
            source_id.remove();
        }
    }
}

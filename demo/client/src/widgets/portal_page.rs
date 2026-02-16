use adw::subclass::prelude::*;
use gtk::{glib, prelude::*};

use super::{Notification, NotificationKind};

mod imp {
    use std::cell::Cell;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, glib::Properties, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/portal_page.ui")]
    #[properties(wrapper_type = super::PortalPage)]
    pub struct PortalPage {
        #[property(get, set)]
        portal_version: Cell<u32>,
        #[template_child]
        pub notification: TemplateChild<Notification>,
        #[template_child]
        pub container: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PortalPage {
        const NAME: &'static str = "PortalPage";
        type Type = super::PortalPage;
        type ParentType = adw::Bin;
        type Interfaces = (gtk::Buildable,);

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_css_name("portal-page");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PortalPage {}
    impl WidgetImpl for PortalPage {
        fn unmap(&self) {
            self.notification.close();

            self.parent_unmap();
        }
    }
    impl BinImpl for PortalPage {}
    impl BuildableImpl for PortalPage {
        fn add_child(&self, builder: &gtk::Builder, child: &glib::Object, type_: Option<&str>) {
            if self.obj().first_child().is_none() {
                self.parent_add_child(builder, child, type_);
            } else {
                // We first check if the main child `box_` has already been bound.
                self.container
                    .append(child.downcast_ref::<gtk::Widget>().unwrap());
            }
        }
    }
}

glib::wrapper! {
    pub struct PortalPage(ObjectSubclass<imp::PortalPage>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

pub trait PortalPageExt {
    fn send_notification(&self, message: &str, kind: NotificationKind);

    fn error(&self, message: &str) {
        self.send_notification(message, NotificationKind::Error);
    }
    fn info(&self, message: &str) {
        self.send_notification(message, NotificationKind::Info);
    }
    fn success(&self, message: &str) {
        self.send_notification(message, NotificationKind::Success);
    }
}

impl<O: IsA<PortalPage>> PortalPageExt for O {
    fn send_notification(&self, message: &str, kind: NotificationKind) {
        self.as_ref().imp().notification.send(message, kind);
    }
}

pub trait PortalPageImpl: BinImpl {}
unsafe impl<T: PortalPageImpl> IsSubclassable<T> for PortalPage {}

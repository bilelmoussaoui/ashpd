use ashpd::zbus;
use ashpd::{RequestProxy, Response, WindowIdentifier};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use adw::subclass::prelude::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/location.ui")]
    pub struct LocationPage {
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LocationPage {
        const NAME: &'static str = "LocationPage";
        type Type = super::LocationPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action(
                "location.locate",
                None,
                move |page, _action, _target| {
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for LocationPage {}
    impl WidgetImpl for LocationPage {}
    impl BinImpl for LocationPage {}
}

glib::wrapper! {
    pub struct LocationPage(ObjectSubclass<imp::LocationPage>) @extends gtk::Widget, adw::Bin;
}

impl LocationPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a LocationPage")
    }
}

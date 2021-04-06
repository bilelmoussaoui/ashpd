use ashpd::zbus;
use ashpd::{RequestProxy, Response, WindowIdentifier};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/device.ui")]
    pub struct DevicePage {}

    #[glib::object_subclass]
    impl ObjectSubclass for DevicePage {
        const NAME: &'static str = "DevicePage";
        type Type = super::DevicePage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("device.request", None, move |page, _action, _target| {
                //page.pick_color().unwrap();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for DevicePage {
        fn constructed(&self, obj: &Self::Type) {
            obj.set_sensitive(!ashpd::is_sandboxed());
        }
    }
    impl WidgetImpl for DevicePage {}
    impl BinImpl for DevicePage {}
}

glib::wrapper! {
    pub struct DevicePage(ObjectSubclass<imp::DevicePage>) @extends gtk::Widget, adw::Bin;
}

impl DevicePage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a DevicePage")
    }
}

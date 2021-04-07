use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/background.ui")]
    pub struct BackgroundPage {}

    impl Default for BackgroundPage {
        fn default() -> Self {
            Self {}
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BackgroundPage {
        const NAME: &'static str = "BackgroundPage";
        type Type = super::BackgroundPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action(
                "background.request",
                None,
                move |_page, _action, _target| {
                    //page.pick_color().unwrap();
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for BackgroundPage {}
    impl WidgetImpl for BackgroundPage {}
    impl BinImpl for BackgroundPage {}
}

glib::wrapper! {
    pub struct BackgroundPage(ObjectSubclass<imp::BackgroundPage>) @extends gtk::Widget, adw::Bin;
}

impl BackgroundPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a BackgroundPage")
    }
}

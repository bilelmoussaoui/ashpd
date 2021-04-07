use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/email.ui")]
    pub struct EmailPage {}

    impl Default for EmailPage {
        fn default() -> Self {
            Self {}
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EmailPage {
        const NAME: &'static str = "EmailPage";
        type Type = super::EmailPage;
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
    impl ObjectImpl for EmailPage {}
    impl WidgetImpl for EmailPage {}
    impl BinImpl for EmailPage {}
}

glib::wrapper! {
    pub struct EmailPage(ObjectSubclass<imp::EmailPage>) @extends gtk::Widget, adw::Bin;
}

impl EmailPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a EmailPage")
    }
}

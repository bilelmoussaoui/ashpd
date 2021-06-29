use ashpd::{desktop::background, WindowIdentifier};
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/background.ui")]
    pub struct BackgroundPage {
        #[template_child]
        pub reason_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub auto_start_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub dbus_activatable_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub command_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub run_bg_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub auto_start_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BackgroundPage {
        const NAME: &'static str = "BackgroundPage";
        type Type = super::BackgroundPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("background.request", None, move |page, _action, _target| {
                page.request_background();
            });
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
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a BackgroundPage")
    }

    fn request_background(&self) {
        let ctx = glib::MainContext::default();
        let self_ = imp::BackgroundPage::from_instance(self);
        let reason = self_.reason_entry.text();
        let auto_start = self_.auto_start_switch.is_active();
        let dbus_activatable = self_.dbus_activatable_switch.is_active();
        let root = self.root().unwrap();

        ctx.spawn_local(clone!(@weak self as page => async move {
            let self_ = imp::BackgroundPage::from_instance(&page);
            let identifier = WindowIdentifier::from_root(&root).await;
            if let Ok(response) = background::request(identifier,
                &reason,
                auto_start,
                Some(self_.command_entry.text().split_whitespace().collect::<Vec<&str>>().as_slice()),
                dbus_activatable).await {

                self_.response_group.show();
                self_.auto_start_label.set_label(&response.run_in_background().to_string());
                self_.run_bg_label.set_label(&response.auto_start().to_string());
            }
        }));
    }
}

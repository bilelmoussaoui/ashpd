use ashpd::{desktop::account, WindowIdentifier};
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk_pixbuf, gio, glib};

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/account.ui")]
    pub struct AccountPage {
        #[template_child]
        pub reason: TemplateChild<gtk::Entry>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub avatar: TemplateChild<gtk::Image>,
        #[template_child]
        pub id_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub name_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AccountPage {
        const NAME: &'static str = "AccountPage";
        type Type = super::AccountPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action(
                "account.information",
                None,
                move |page, _action, _target| {
                    page.fetch_user_information();
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for AccountPage {}
    impl WidgetImpl for AccountPage {}
    impl BinImpl for AccountPage {}
}

glib::wrapper! {
    pub struct AccountPage(ObjectSubclass<imp::AccountPage>) @extends gtk::Widget, adw::Bin;
}

impl AccountPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a AccountPage")
    }

    pub fn fetch_user_information(&self) {
        let ctx = glib::MainContext::default();
        let root = self.native().unwrap();
        ctx.spawn_local(clone!(@weak self as page => async move {
            let self_ = imp::AccountPage::from_instance(&page);
            let identifier = WindowIdentifier::from_native(&root).await;
            let reason = self_.reason.text();

            if let Ok(user_info) = account::user_information(&identifier, &reason).await
            {
                self_.id_label.set_text(user_info.id());
                self_.name_label.set_text(user_info.name());
                let path: std::path::PathBuf = user_info.image().trim_start_matches("file://").into();
                let pixbuf = gdk_pixbuf::Pixbuf::from_file(path).unwrap();

                self_.avatar.set_from_pixbuf(Some(&pixbuf));
                self_.response_group.show();
            }
        }));
    }
}

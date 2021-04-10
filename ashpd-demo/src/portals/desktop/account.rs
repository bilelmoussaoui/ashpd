use ashpd::{desktop::account, Response, WindowIdentifier};
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

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
                    page.get_user_information();
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
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a AccountPage")
    }

    pub fn get_user_information(&self) {
        let self_ = imp::AccountPage::from_instance(self);
        let reason = self_.reason.get_text();
        let ctx = glib::MainContext::default();
        let id_label = self_.id_label.get();
        let name_label = self_.name_label.get();
        let response_group = self_.response_group.get();
        let avatar = self_.avatar.get();
        let root = self.get_root().unwrap();
        ctx.spawn_local(clone!(@weak id_label, @weak name_label => async move {
            let identifier = WindowIdentifier::from_window(&root).await;
            if let Ok(Response::Ok(user_info)) = account::get_user_information(identifier, &reason).await
            {
                id_label.set_text(&user_info.id);
                name_label.set_text(&user_info.name);
                let file = gio::File::new_for_uri(&user_info.image);
                let icon = gio::FileIcon::new(&file);
                avatar.set_from_gicon(&icon);
                response_group.show();
            }
        }));
    }
}

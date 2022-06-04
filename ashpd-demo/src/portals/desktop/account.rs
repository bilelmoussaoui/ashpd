use ashpd::{desktop::account, WindowIdentifier};
use glib::clone;
use gtk::{gdk_pixbuf, glib, prelude::*, subclass::prelude::*};

use crate::widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl};

mod imp {
    use adw::subclass::prelude::*;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
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
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action(
                "account.information",
                None,
                move |page, _action, _target| {
                    let ctx = glib::MainContext::default();
                    ctx.spawn_local(clone!(@weak page => async move {
                        page.fetch_user_information().await;
                    }));
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
    impl PortalPageImpl for AccountPage {}
}

glib::wrapper! {
    pub struct AccountPage(ObjectSubclass<imp::AccountPage>) @extends gtk::Widget, adw::Bin, PortalPage;
}

impl AccountPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a AccountPage")
    }

    async fn fetch_user_information(&self) {
        let root = self.native().unwrap();
        let imp = self.imp();
        let identifier = WindowIdentifier::from_native(&root).await;
        let reason = imp.reason.text();
        self.send_notification("Fetching user information...", NotificationKind::Info);

        match account::user_information(&identifier, &reason).await {
            Ok(user_info) => {
                self.send_notification(
                    "User information request was successful",
                    NotificationKind::Success,
                );
                imp.id_label.set_text(user_info.id());
                imp.name_label.set_text(user_info.name());
                let path: std::path::PathBuf =
                    user_info.image().trim_start_matches("file://").into();
                let pixbuf = gdk_pixbuf::Pixbuf::from_file(path).unwrap();

                imp.avatar.set_from_pixbuf(Some(&pixbuf));
                imp.response_group.show();
            }
            Err(_err) => {
                self.send_notification(
                    "Request to fetch user information failed",
                    NotificationKind::Error,
                );
            }
        }
    }
}

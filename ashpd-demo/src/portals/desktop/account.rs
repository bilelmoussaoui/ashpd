use ashpd::{desktop::account::UserInformation, WindowIdentifier};
use gtk::{gdk, glib, prelude::*, subclass::prelude::*};

use crate::widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl};

mod imp {
    use adw::subclass::prelude::*;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/account.ui")]
    pub struct AccountPage {
        #[template_child]
        pub reason_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub avatar: TemplateChild<adw::Avatar>,
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
            klass.bind_template();
            klass.install_action_async("account.information", None, |page, _, _| async move {
                page.fetch_user_information().await;
            });
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
    pub struct AccountPage(ObjectSubclass<imp::AccountPage>)
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl AccountPage {
    async fn fetch_user_information(&self) {
        let root = self.native().unwrap();
        let imp = self.imp();
        let identifier = WindowIdentifier::from_native(&root).await;
        let reason = imp.reason_row.text();
        self.send_notification("Fetching user information...", NotificationKind::Info);
        let request = UserInformation::request()
            .identifier(identifier)
            .reason(&*reason);
        match request.send().await.and_then(|r| r.response()) {
            Ok(user_info) => {
                self.send_notification(
                    "User information request was successful",
                    NotificationKind::Success,
                );
                imp.id_label.set_text(user_info.id());
                imp.name_label.set_text(user_info.name());
                match user_info
                    .image()
                    .to_file_path()
                    .map_err(|_| {
                        glib::Error::new(glib::FileError::Failed, "Failed to retrieve file path")
                    })
                    .and_then(gdk::Texture::from_filename)
                {
                    Ok(texture) => {
                        imp.avatar.set_custom_image(Some(&texture));
                        imp.avatar.set_visible(true);
                    }
                    Err(err) => {
                        tracing::error!("Failed to set user avatar {err}");
                        imp.avatar.set_visible(false);
                    }
                };
                imp.response_group.set_visible(true);
            }
            Err(_err) => {
                self.send_notification(
                    "Request to fetch user information failed",
                    NotificationKind::Error,
                );
            }
        };
    }
}

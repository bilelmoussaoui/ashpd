use adw::subclass::prelude::*;
use ashpd::{
    WindowIdentifier,
    desktop::account::{AccountProxy, UserInformation},
};
use gtk::{gdk, glib, prelude::*};

use crate::{
    portals::spawn_tokio,
    widgets::{PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
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
    impl WidgetImpl for AccountPage {
        fn map(&self) {
            self.parent_map();
            let obj = self.obj();

            glib::spawn_future_local(glib::clone!(
                #[weak]
                obj,
                async move {
                    if let Ok(proxy) = spawn_tokio(async { AccountProxy::new().await }).await {
                        obj.set_property("portal-version", proxy.version());
                    }
                }
            ));
        }
    }
    impl BinImpl for AccountPage {}
    impl PortalPageImpl for AccountPage {}
}

glib::wrapper! {
    pub struct AccountPage(ObjectSubclass<imp::AccountPage>)
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl AccountPage {
    async fn fetch_user_information(&self) {
        let root = self.native().unwrap();
        let imp = self.imp();
        let identifier = WindowIdentifier::from_native(&root).await;
        let reason = imp.reason_row.text();
        self.info("Fetching user information...");
        let response = spawn_tokio(async move {
            let request = UserInformation::request()
                .identifier(identifier)
                .reason(&*reason);
            request.send().await.and_then(|r| r.response())
        })
        .await;
        match response {
            Ok(user_info) => {
                self.success("User information request was successful");
                imp.id_label.set_text(user_info.id());
                imp.name_label.set_text(user_info.name());
                match glib::Uri::try_from(user_info.image())
                    .ok()
                    .map(|uri| uri.path())
                    .ok_or_else(|| {
                        glib::Error::new(glib::FileError::Failed, "Failed to retrieve file path")
                    })
                    .and_then(gdk::Texture::from_filename)
                {
                    Ok(texture) => {
                        imp.avatar.set_custom_image(Some(&texture));
                        imp.avatar.set_visible(true);
                    }
                    Err(err) => {
                        tracing::error!("Failed to load user avatar: {err}");
                        imp.avatar.set_visible(false);
                    }
                };
                imp.response_group.set_visible(true);
            }
            Err(err) => {
                tracing::error!("Failed to retrieve user information: {err}");
                self.error(&format!("Request to fetch user information failed: {err}"));
            }
        };
    }
}

use adw::subclass::prelude::*;
use ashpd::{WindowIdentifier, desktop::background::Background};
use gtk::{glib, prelude::*};

use crate::{
    portals::{is_empty, spawn_tokio},
    widgets::{PortalPage, PortalPageExt, PortalPageImpl},
};
mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/background.ui")]
    pub struct BackgroundPage {
        #[template_child]
        pub reason_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub auto_start_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub dbus_activatable_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub command_entry: TemplateChild<adw::EntryRow>,
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
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.install_action_async("background.request", None, |page, _, _| async move {
                page.request_background().await;
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for BackgroundPage {}
    impl WidgetImpl for BackgroundPage {
        fn map(&self) {
            self.parent_map();
            let obj = self.obj();

            glib::spawn_future_local(glib::clone!(
                #[weak]
                obj,
                async move {
                    if let Ok(proxy) = spawn_tokio(async {
                        ashpd::desktop::background::BackgroundProxy::new().await
                    })
                    .await
                    {
                        obj.set_property("portal-version", proxy.version());
                    }
                }
            ));
        }
    }
    impl BinImpl for BackgroundPage {}
    impl PortalPageImpl for BackgroundPage {}
}

glib::wrapper! {
    pub struct BackgroundPage(ObjectSubclass<imp::BackgroundPage>)
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl BackgroundPage {
    async fn request_background(&self) {
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let imp = self.imp();
        let reason = imp.reason_entry.text();
        let auto_start = imp.auto_start_switch.is_active();
        let dbus_activatable = imp.dbus_activatable_switch.is_active();
        let command = is_empty(imp.command_entry.text())
            .map(|txt| {
                txt.split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        self.info("Requesting background access");

        let response = spawn_tokio(async move {
            let request = Background::request()
                .identifier(identifier)
                .reason(&*reason)
                .auto_start(auto_start)
                .dbus_activatable(dbus_activatable)
                .command::<Vec<_>, String>(command);
            request.send().await.and_then(|r| r.response())
        })
        .await;

        match response {
            Ok(response) => {
                imp.response_group.set_visible(true);
                imp.auto_start_label
                    .set_label(&response.auto_start().to_string());
                imp.run_bg_label
                    .set_label(&response.run_in_background().to_string());
                self.success("Background request was successful");
            }
            Err(err) => {
                tracing::error!("Failed to request running in background: {err}");
                self.error("Request to run in background failed");
            }
        }
    }
}

impl Default for BackgroundPage {
    fn default() -> Self {
        glib::Object::new()
    }
}

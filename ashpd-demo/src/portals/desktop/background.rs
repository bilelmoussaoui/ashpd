use ashpd::{desktop::background::Background, WindowIdentifier};
use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::{
    portals::is_empty,
    widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl},
};
mod imp {
    use adw::subclass::prelude::*;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/background.ui")]
    pub struct BackgroundPage {
        #[template_child]
        pub reason_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub auto_start_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub dbus_activatable_switch: TemplateChild<gtk::Switch>,
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
            klass.install_action_async(
                "background.request",
                None,
                move |page, _action, _target| async move {
                    page.request_background().await;
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
    impl PortalPageImpl for BackgroundPage {}
}

glib::wrapper! {
    pub struct BackgroundPage(ObjectSubclass<imp::BackgroundPage>)
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl BackgroundPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new()
    }

    async fn request_background(&self) {
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;
        let imp = self.imp();
        let reason = imp.reason_entry.text();
        let auto_start = imp.auto_start_switch.is_active();
        let dbus_activatable = imp.dbus_activatable_switch.is_active();
        let command = is_empty(imp.command_entry.text()).map(|txt| {
            txt.split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        });

        let request = Background::builder()
            .identifier(identifier)
            .reason(&*reason)
            .auto_start(auto_start)
            .dbus_activatable(dbus_activatable)
            .command::<Vec<_>, String>(command);

        self.send_notification("Requesting background access", NotificationKind::Info);

        match request.build().await.and_then(|r| r.response()) {
            Ok(response) => {
                imp.response_group.show();
                imp.auto_start_label
                    .set_label(&response.auto_start().to_string());
                imp.run_bg_label
                    .set_label(&response.run_in_background().to_string());
                self.send_notification(
                    "Background request was successful",
                    NotificationKind::Success,
                );
            }
            Err(_err) => {
                self.send_notification(
                    "Request to run in background failed",
                    NotificationKind::Error,
                );
            }
        }
    }
}

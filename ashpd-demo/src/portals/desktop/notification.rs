use crate::widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl};
use adw::prelude::*;
use ashpd::{
    desktop::notification::{Button, Notification, NotificationProxy, Priority},
    zbus,
    zvariant::Value,
};
use glib::clone;
use gtk::glib;
use gtk::subclass::prelude::*;

mod imp {
    use adw::subclass::prelude::*;
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/notification.ui")]
    pub struct NotificationPage {
        #[template_child]
        pub id_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub title_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub body_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub priority_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub id_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub action_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub parameters_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NotificationPage {
        const NAME: &'static str = "NotificationPage";
        type Type = super::NotificationPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("notification.send", None, move |page, _action, _target| {
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak page => async move {
                    if let Err(err) = page.send().await {
                        tracing::error!("Failed to send a notification {}", err);
                    }
                }));
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for NotificationPage {}
    impl WidgetImpl for NotificationPage {}
    impl BinImpl for NotificationPage {}
    impl PortalPageImpl for NotificationPage {}
}

glib::wrapper! {
    pub struct NotificationPage(ObjectSubclass<imp::NotificationPage>) @extends gtk::Widget, adw::Bin, PortalPage;
}

impl NotificationPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a NotificationPage")
    }

    async fn send(&self) -> ashpd::Result<()> {
        let imp = self.imp();

        let notification_id = imp.id_entry.text();
        let title = imp.title_entry.text();
        let body = imp.body_entry.text();
        let priority = match imp.priority_combo.selected() {
            0 => Priority::Low,
            1 => Priority::Normal,
            2 => Priority::High,
            3 => Priority::Urgent,
            _ => unimplemented!(),
        };

        let notification = Notification::new(&title)
            .default_action("open")
            .default_action_target(Value::U32(100).into())
            .body(&body)
            .priority(priority)
            .button(Button::new("Copy", "copy").target(Value::U32(32).into()))
            .button(Button::new("Delete", "delete").target(Value::U32(40).into()));

        let cnx = zbus::Connection::session().await?;
        let proxy = NotificationProxy::new(&cnx).await?;
        match proxy.add_notification(&notification_id, notification).await {
            Ok(_) => {
                self.send_notification("Notification sent", NotificationKind::Success);
                let action = proxy.receive_action_invoked().await?;
                self.send_notification(
                    &format!("User interacted with notification \"{}\"", notification_id),
                    NotificationKind::Info,
                );

                imp.response_group.show();
                imp.id_label.set_text(action.id());
                imp.action_name_label.set_text(action.name());
                imp.parameters_label
                    .set_text(&format!("{:#?}", action.parameter()));
            }
            Err(_) => {
                self.send_notification("Failed to send a notification", NotificationKind::Error);
            }
        }
        Ok(())
    }
}

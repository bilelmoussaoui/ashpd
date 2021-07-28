use std::str::FromStr;

use adw::prelude::*;
use ashpd::{
    desktop::notification::{Action, Button, Notification, NotificationProxy, Priority},
    zbus,
    zvariant::Value,
};
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk_macros::spawn;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
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
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("notification.send", None, move |page, _action, _target| {
                page.send_notification().unwrap();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for NotificationPage {
        fn constructed(&self, _obj: &Self::Type) {
            let model = gtk::StringList::new(&[
                &Priority::Low.to_string(),
                &Priority::Normal.to_string(),
                &Priority::High.to_string(),
                &Priority::Urgent.to_string(),
            ]);
            self.priority_combo.set_model(Some(&model));
            self.priority_combo.set_selected(Priority::Normal as u32);
        }
    }
    impl WidgetImpl for NotificationPage {}
    impl BinImpl for NotificationPage {}
}

glib::wrapper! {
    pub struct NotificationPage(ObjectSubclass<imp::NotificationPage>) @extends gtk::Widget, adw::Bin;
}

impl NotificationPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a NotificationPage")
    }

    pub fn send_notification(&self) -> zbus::fdo::Result<()> {
        let self_ = imp::NotificationPage::from_instance(self);

        let notification_id = self_.id_entry.text();
        let title = self_.title_entry.text();
        let body = self_.body_entry.text();
        let selected_item = self_
            .priority_combo
            .selected_item()
            .unwrap()
            .downcast::<gtk::StringObject>()
            .unwrap()
            .string();
        let priority = Priority::from_str(&selected_item).unwrap();

        spawn!(clone!(@weak self as page => async move {
            let self_ = imp::NotificationPage::from_instance(&page);
            let action = notify(
                &notification_id,
                Notification::new(&title)
                    .default_action("open")
                    .default_action_target(Value::U32(100).into())
                    .body(&body)
                    .priority(priority)
                    .button(Button::new("Copy", "copy").target(Value::U32(32).into()))
                    .button(Button::new("Delete", "delete").target(Value::U32(40).into())),
            )
            .await
            .unwrap();
            self_.response_group.show();
            self_.id_label.set_text(action.id());
            self_.action_name_label.set_text(action.name());
            self_.parameters_label.set_text(&format!("{:#?}", action.parameter()));
        }));

        Ok(())
    }
}

async fn notify(id: &str, notification: Notification) -> ashpd::Result<Action> {
    let cnx = zbus::azync::Connection::new_session().await?;
    let proxy = NotificationProxy::new(&cnx).await?;
    proxy.add_notification(&id, notification).await?;
    let action = proxy.receive_action_invoked().await?;
    Ok(action)
}

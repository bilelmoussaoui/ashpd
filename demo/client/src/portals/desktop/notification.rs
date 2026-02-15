use adw::{prelude::*, subclass::prelude::*};
use ashpd::desktop::{
    Icon,
    notification::{Button, Notification, NotificationProxy, Priority},
};
use futures_util::stream::StreamExt;
use gtk::glib;

use self::button::NotificationButton;
use crate::{
    portals::spawn_tokio,
    widgets::{PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/notification.ui")]
    pub struct NotificationPage {
        #[template_child]
        pub id_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub title_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub body_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub markup_body_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub icon_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub priority_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub default_action_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub default_action_target_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub id_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub action_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub parameters_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub buttons_box: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NotificationPage {
        const NAME: &'static str = "NotificationPage";
        type Type = super::NotificationPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async("notification.send", None, |page, _, _| async move {
                if let Err(err) = page.send().await {
                    tracing::error!("Failed to send a notification {}", err);
                }
            });
            klass.install_action("notification.add_button", None, |page, _, _| {
                page.add_button();
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
    pub struct NotificationPage(ObjectSubclass<imp::NotificationPage>)
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl NotificationPage {
    fn add_button(&self) {
        let button = NotificationButton::default();
        button.connect_removed(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |button| {
                page.imp().buttons_box.remove(button);
            }
        ));
        self.imp().buttons_box.append(&button);
    }

    fn buttons(&self) -> Vec<Button> {
        let mut buttons = vec![];
        let mut child = match self.imp().buttons_box.first_child() {
            Some(t) => t,
            None => return buttons,
        };

        loop {
            let button_widget = child.downcast_ref::<NotificationButton>().unwrap();
            buttons.push(button_widget.button());

            if let Some(next_child) = child.next_sibling() {
                child = next_child;
            } else {
                break;
            }
        }
        buttons
    }

    async fn send(&self) -> ashpd::Result<()> {
        let imp = self.imp();

        let notification_id = imp.id_entry.text();
        let title = imp.title_entry.text();
        let body = imp.body_entry.text();
        let markup_body = imp.markup_body_entry.text();
        let icon_name = imp.icon_entry.text();
        let default_action = imp.default_action_entry.text();
        let default_action_target = imp.default_action_target_entry.text();
        let priority = match imp.priority_combo.selected() {
            0 => Priority::Low,
            1 => Priority::Normal,
            2 => Priority::High,
            3 => Priority::Urgent,
            _ => unimplemented!(),
        };

        let mut notification = Notification::new(&title)
            .default_action(&*default_action)
            .default_action_target(&*default_action_target)
            .body(&*body)
            .priority(priority);

        if !markup_body.is_empty() {
            notification = notification.markup_body(&*markup_body);
        }
        if !icon_name.is_empty() {
            notification = notification.icon(Icon::with_names(&[&icon_name]));
        }

        for button in self.buttons().into_iter() {
            notification = notification.button(button);
        }

        let notification_id_owned = notification_id.to_owned();
        let response: ashpd::Result<NotificationProxy> = spawn_tokio(async move {
            let proxy = NotificationProxy::new().await?;
            proxy
                .add_notification(&notification_id_owned, notification)
                .await?;
            Ok(proxy)
        })
        .await;
        match response {
            Ok(proxy) => {
                self.success("Notification sent");
                let action = spawn_tokio(async move {
                    let action = proxy
                        .receive_action_invoked()
                        .await?
                        .next()
                        .await
                        .expect("Stream exhausted");
                    ashpd::Result::Ok(action)
                })
                .await?;
                self.info(&format!(
                    "User interacted with notification \"{notification_id}\""
                ));

                imp.response_group.set_visible(true);
                imp.id_label.set_text(action.id());
                imp.action_name_label.set_text(action.name());
                imp.parameters_label
                    .set_text(&action.parameter()[0].downcast_ref::<String>().unwrap());
            }
            Err(err) => {
                tracing::error!("Failed to send a notification: {err}");
                self.error("Failed to send a notification");
            }
        }
        Ok(())
    }
}

mod button {
    use super::*;
    mod imp {
        use std::sync::OnceLock;

        use adw::subclass::prelude::BinImpl;
        use glib::subclass::Signal;

        use super::*;

        #[derive(Debug, Default)]
        pub struct NotificationButton {
            pub(super) label_row: adw::EntryRow,
            pub(super) action_row: adw::EntryRow,
            pub(super) target_row: adw::EntryRow,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for NotificationButton {
            const NAME: &'static str = "NotificationButton";
            type Type = super::NotificationButton;
            type ParentType = adw::Bin;
        }

        impl ObjectImpl for NotificationButton {
            fn signals() -> &'static [Signal] {
                static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
                SIGNALS.get_or_init(|| vec![Signal::builder("removed").action().build()])
            }

            fn constructed(&self) {
                self.parent_constructed();
                self.obj().create_widgets();
            }
        }
        impl WidgetImpl for NotificationButton {}
        impl BinImpl for NotificationButton {}
    }

    glib::wrapper! {
        pub struct NotificationButton(ObjectSubclass<imp::NotificationButton>)
            @extends gtk::Widget, adw::Bin,
            @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
    }

    impl NotificationButton {
        pub fn connect_removed<F>(&self, callback: F) -> glib::SignalHandlerId
        where
            F: Fn(&Self) + 'static,
        {
            self.connect_closure(
                "removed",
                false,
                glib::closure_local!(move |obj: &Self| {
                    callback(obj);
                }),
            )
        }

        pub fn button(&self) -> Button {
            let imp = self.imp();
            let label = imp.label_row.text();
            let action = imp.action_row.text();
            let target = imp.target_row.text();
            Button::new(&label, &action).target(&*target)
        }

        fn create_widgets(&self) {
            let imp = self.imp();
            let container = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .build();

            let list_box = gtk::ListBox::new();
            list_box.add_css_class("boxed-list");

            imp.label_row.set_title("Label");
            list_box.append(&imp.label_row);
            imp.action_row.set_title("Action");
            list_box.append(&imp.action_row);
            imp.target_row.set_title("Action Target");
            list_box.append(&imp.target_row);

            container.append(&list_box);

            let remove_button = gtk::Button::builder()
                .halign(gtk::Align::End)
                .valign(gtk::Align::Center)
                .margin_top(6)
                .label("Remove")
                .margin_bottom(12)
                .build();
            remove_button.add_css_class("destructive-action");
            remove_button.connect_clicked(glib::clone!(
                #[weak(rename_to = obj)]
                self,
                move |_btn| {
                    obj.emit_by_name::<()>("removed", &[]);
                }
            ));
            container.append(&remove_button);

            self.set_child(Some(&container));
        }
    }
    impl Default for NotificationButton {
        fn default() -> Self {
            glib::Object::new()
        }
    }
}

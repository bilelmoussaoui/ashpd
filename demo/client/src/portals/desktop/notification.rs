use std::{fs::File, os::fd::OwnedFd};

use adw::{prelude::*, subclass::prelude::*};
use ashpd::desktop::{
    Icon,
    notification::{Button, Category, DisplayHint, Notification, NotificationProxy, Priority},
};
use futures_util::stream::StreamExt;
use gtk::{gio, glib};

use self::button::NotificationButton;
use crate::{
    portals::spawn_tokio,
    widgets::{PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
    use std::cell::RefCell;

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
        pub category_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub transient_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub tray_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub persistent_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub hide_on_lockscreen_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub hide_content_on_lockscreen_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub show_as_new_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub sound_row: TemplateChild<adw::ActionRow>,
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
        pub selected_sound: RefCell<Option<gio::File>>,
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
            klass.install_action_async(
                "notification.select_sound",
                None,
                |page, _, _| async move {
                    if let Err(err) = page.select_sound().await {
                        tracing::error!("Failed to select sound file {err}");
                    }
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for NotificationPage {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Add default buttons
            let button1 = NotificationButton::default();
            button1.imp().label_row.set_text("View");
            button1.imp().action_row.set_text("app.view-notification");
            button1.imp().target_row.set_text("notification-id");
            button1.connect_removed(glib::clone!(
                #[weak]
                obj,
                move |button| {
                    obj.imp().buttons_box.remove(button);
                }
            ));
            self.buttons_box.append(&button1);

            let button2 = NotificationButton::default();
            button2.imp().label_row.set_text("Dismiss");
            button2
                .imp()
                .action_row
                .set_text("app.dismiss-notification");
            button2.imp().target_row.set_text("notification-id");
            button2.connect_removed(glib::clone!(
                #[weak]
                obj,
                move |button| {
                    obj.imp().buttons_box.remove(button);
                }
            ));
            self.buttons_box.append(&button2);
        }
    }
    impl WidgetImpl for NotificationPage {
        fn map(&self) {
            self.parent_map();
            let obj = self.obj();

            glib::spawn_future_local(glib::clone!(
                #[weak]
                obj,
                async move {
                    if let Ok(proxy) = spawn_tokio(async { NotificationProxy::new().await }).await {
                        obj.set_property("portal-version", proxy.version());
                    }
                }
            ));
        }
    }
    impl BinImpl for NotificationPage {}
    impl PortalPageImpl for NotificationPage {}
}

glib::wrapper! {
    pub struct NotificationPage(ObjectSubclass<imp::NotificationPage>)
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl NotificationPage {
    async fn select_sound(&self) -> Result<(), glib::Error> {
        let imp = self.imp();
        let filter = gtk::FileFilter::new();
        filter.add_mime_type("audio/*");
        filter.set_name(Some("Audio files"));

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);

        let root = self.native().unwrap();
        let file = gtk::FileDialog::builder()
            .accept_label("Select")
            .modal(true)
            .title("Notification Sound")
            .filters(&filters)
            .build()
            .open_future(root.downcast_ref::<gtk::Window>())
            .await?;

        imp.sound_row.set_subtitle(&file.uri());
        imp.selected_sound.replace(Some(file));
        Ok(())
    }

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
        let category = match imp.category_combo.selected() {
            0 => None, // None
            1 => Some(Category::ImMessage),
            2 => Some(Category::AlarmRinging),
            3 => Some(Category::IncomingCall),
            4 => Some(Category::OngoingCall),
            5 => Some(Category::MissedCall),
            6 => Some(Category::ExtremeWeather),
            7 => Some(Category::CellNetworkExtremeDanger),
            8 => Some(Category::CellNetworkSevereDanger),
            9 => Some(Category::CellNetworkAmberAlert),
            10 => Some(Category::CellNetworkBroadcastTest),
            11 => Some(Category::LowBattery),
            12 => Some(Category::WebNotification),
            _ => None,
        };
        let mut display_hints = Vec::new();
        if imp.transient_switch.is_active() {
            display_hints.push(DisplayHint::Transient);
        }
        if imp.tray_switch.is_active() {
            display_hints.push(DisplayHint::Tray);
        }
        if imp.persistent_switch.is_active() {
            display_hints.push(DisplayHint::Persistent);
        }
        if imp.hide_on_lockscreen_switch.is_active() {
            display_hints.push(DisplayHint::HideOnLockScreen);
        }
        if imp.hide_content_on_lockscreen_switch.is_active() {
            display_hints.push(DisplayHint::HideContentOnLockScreen);
        }
        if imp.show_as_new_switch.is_active() {
            display_hints.push(DisplayHint::ShowAsNew);
        }
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
            notification = notification.icon(Icon::with_names([&icon_name]));
        }
        if let Some(cat) = category {
            notification = notification.category(cat);
        }
        if !display_hints.is_empty() {
            notification = notification.display_hint(display_hints);
        }

        let selected_sound = imp.selected_sound.borrow().clone();
        if let Some(sound_file) = selected_sound {
            let path = sound_file.path().unwrap();
            if let Ok(file) = File::open(path) {
                notification = notification.sound(OwnedFd::from(file));
            }
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
            pub label_row: adw::EntryRow,
            pub action_row: adw::EntryRow,
            pub target_row: adw::EntryRow,
            pub purpose_combo: adw::ComboRow,
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
            let purpose = match imp.purpose_combo.selected() {
                0 => None, // None
                1 => Some(ashpd::desktop::notification::ButtonPurpose::ImReplyWithText),
                2 => Some(ashpd::desktop::notification::ButtonPurpose::CallAccept),
                3 => Some(ashpd::desktop::notification::ButtonPurpose::CallDecline),
                4 => Some(ashpd::desktop::notification::ButtonPurpose::CallHangup),
                5 => Some(ashpd::desktop::notification::ButtonPurpose::CallEnableSpeakerphone),
                6 => Some(ashpd::desktop::notification::ButtonPurpose::CallDisableSpeakerphone),
                7 => Some(ashpd::desktop::notification::ButtonPurpose::SystemCustomAlert),
                _ => None,
            };
            let mut button = Button::new(&label, &action).target(&*target);
            if let Some(p) = purpose {
                button = button.purpose(p);
            }
            button
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

            imp.purpose_combo.set_title("Purpose");
            imp.purpose_combo.set_model(Some(&gtk::StringList::new(&[
                "None",
                "IM Reply with Text",
                "Call Accept",
                "Call Decline",
                "Call Hang Up",
                "Call Enable Speakerphone",
                "Call Disable Speakerphone",
                "System Custom Alert",
            ])));
            list_box.append(&imp.purpose_combo);

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

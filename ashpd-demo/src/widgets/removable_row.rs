use adw::{prelude::*, subclass::prelude::*};
use gtk::glib;

mod imp {
    use glib::subclass::Signal;
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug, Default)]
    pub struct RemovableRow {}

    #[glib::object_subclass]
    impl ObjectSubclass for RemovableRow {
        const NAME: &'static str = "RemovableRow";
        type Type = super::RemovableRow;
        type ParentType = adw::ActionRow;
    }

    impl ObjectImpl for RemovableRow {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("removed", &[], <()>::static_type().into())
                    .action()
                    .build()]
            });
            SIGNALS.as_ref()
        }

        fn constructed(&self, obj: &Self::Type) {
            let remove_button = gtk::Button::from_icon_name("edit-delete-symbolic");
            remove_button.set_valign(gtk::Align::Center);
            remove_button.add_css_class("circular");
            remove_button.add_css_class("flat");
            obj.add_suffix(&remove_button);

            remove_button.connect_clicked(glib::clone!(@weak obj => move |_btn| {
                obj.emit_by_name::<()>("removed", &[]);
            }));
            obj.set_activatable(true);
            obj.set_activatable_widget(Some(&remove_button));
            self.parent_constructed(obj);
        }
    }
    impl WidgetImpl for RemovableRow {}
    impl ListBoxRowImpl for RemovableRow {}
    impl PreferencesRowImpl for RemovableRow {}
    impl ActionRowImpl for RemovableRow {}
}

glib::wrapper! {
    pub struct RemovableRow(ObjectSubclass<imp::RemovableRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow;
}

impl RemovableRow {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a RemovableRow")
    }

    pub fn connect_removed<F>(&self, callback: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_local("removed", false, move |args| {
            let obj = args.get(0).unwrap().get::<Self>().unwrap();
            callback(&obj);
            None
        })
    }
}

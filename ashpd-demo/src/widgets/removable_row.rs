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
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("removed").action().build()]);
            SIGNALS.as_ref()
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
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
}

impl Default for RemovableRow {
    fn default() -> Self {
        glib::Object::new(&[])
    }
}

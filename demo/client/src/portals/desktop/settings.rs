use adw::{prelude::*, subclass::prelude::*};
use ashpd::{
    desktop::settings::{APPEARANCE_NAMESPACE, ColorScheme, Contrast, Settings},
    zvariant::Value,
};
use gtk::{
    gdk,
    glib::{self, clone},
};

use crate::{
    portals::spawn_tokio,
    widgets::{ColorWidget, PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Default, Debug, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/settings.ui")]
    pub struct SettingsPage {
        #[template_child]
        pub color_scheme_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub accent_color_widget: TemplateChild<ColorWidget>,
        #[template_child]
        pub contrast_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub all_settings_group: TemplateChild<adw::PreferencesGroup>,

        pub stream_handles: RefCell<Vec<glib::JoinHandle<()>>>,
        pub settings: RefCell<Option<Settings>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsPage {
        const NAME: &'static str = "SettingsPage";
        type Type = super::SettingsPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SettingsPage {}

    impl WidgetImpl for SettingsPage {
        fn map(&self) {
            let widget = self.obj();
            glib::spawn_future_local(clone!(
                #[weak]
                widget,
                async move {
                    if let Err(err) = widget.load_settings().await {
                        tracing::error!("Failed to load settings: {err}");
                        widget.error(&format!("Failed to load settings: {err}."));
                    }
                }
            ));
            self.parent_map();
        }

        fn unmap(&self) {
            for handle in self.stream_handles.take() {
                handle.abort();
            }
            self.settings.take();
            self.parent_unmap();
        }
    }

    impl BinImpl for SettingsPage {}
    impl PortalPageImpl for SettingsPage {}
}

glib::wrapper! {
    pub struct SettingsPage(ObjectSubclass<imp::SettingsPage>)
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl SettingsPage {
    async fn load_settings(&self) -> ashpd::Result<()> {
        let (settings, color_scheme, accent_color, contrast, all_settings) =
            spawn_tokio(async move {
                let settings = Settings::new().await?;
                let color_scheme = settings.color_scheme().await?;
                let accent_color = settings.accent_color().await?;
                let contrast = settings.contrast().await?;
                let all_settings = settings.read_all(&[""]).await?;
                ashpd::Result::Ok((settings, color_scheme, accent_color, contrast, all_settings))
            })
            .await?;

        let imp = self.imp();
        imp.color_scheme_label
            .set_text(&format_color_scheme(color_scheme));
        imp.accent_color_widget
            .set_rgba(gdk::RGBA::from(accent_color));
        imp.contrast_label.set_text(&format_contrast(contrast));

        // Display all settings
        for (namespace, settings_map) in all_settings {
            // Skip appearance namespace as we handle those with dedicated getters
            if namespace == APPEARANCE_NAMESPACE {
                continue;
            }

            for (key, value) in settings_map {
                let row = adw::ActionRow::builder()
                    .title(format!("{}.{}", namespace, key))
                    .build();

                let formatted_value = format_value(&value);

                let value_label = gtk::Label::builder()
                    .label(formatted_value)
                    .halign(gtk::Align::Start)
                    .valign(gtk::Align::Center)
                    .selectable(true)
                    .wrap(true)
                    .max_width_chars(50)
                    .xalign(0.0)
                    .build();
                value_label.add_css_class("dim-label");
                row.add_suffix(&value_label);

                imp.all_settings_group.add(&row);
            }
        }

        imp.settings.replace(Some(settings));

        self.success("Settings loaded successfully");
        Ok(())
    }
}

fn format_color_scheme(scheme: ColorScheme) -> String {
    match scheme {
        ColorScheme::NoPreference => "No Preference".to_string(),
        ColorScheme::PreferDark => "Dark".to_string(),
        ColorScheme::PreferLight => "Light".to_string(),
    }
}

fn format_contrast(contrast: Contrast) -> String {
    match contrast {
        Contrast::NoPreference => "No Preference".to_string(),
        Contrast::High => "High".to_string(),
    }
}

fn format_value(value: &ashpd::zvariant::OwnedValue) -> String {
    if let Ok(inner) = value.downcast_ref::<Value>() {
        match inner {
            Value::U8(v) => v.to_string(),
            Value::Bool(v) => v.to_string(),
            Value::I16(v) => v.to_string(),
            Value::U16(v) => v.to_string(),
            Value::I32(v) => v.to_string(),
            Value::U32(v) => v.to_string(),
            Value::I64(v) => v.to_string(),
            Value::U64(v) => v.to_string(),
            Value::F64(v) => v.to_string(),
            Value::Str(v) => v.to_string(),
            Value::Signature(v) => v.to_string(),
            Value::ObjectPath(v) => v.to_string(),
            Value::Value(v) => format_value(&v.try_to_owned().unwrap_or_else(|_| value.clone())),
            Value::Array(_) | Value::Dict(_) | Value::Structure(_) | Value::Fd(_) => {
                format!("{:?}", inner)
            }
        }
    } else {
        // Fall back to debug representation
        format!("{:?}", value)
    }
}

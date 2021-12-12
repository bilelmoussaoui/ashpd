use gtk::glib;
use gtk::prelude::*;

mod imp {
    use std::cell::RefCell;

    use glib::{ParamFlags, ParamSpec, ParamSpecString, Value};
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/sidebar_row.ui")]
    pub struct SidebarRow {
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        pub name: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarRow {
        const NAME: &'static str = "SidebarRow";
        type Type = super::SidebarRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for SidebarRow {
        fn properties() -> &'static [ParamSpec] {
            static PROPS: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new(
                        "label",
                        "Label",
                        "Row Label",
                        Some(""),
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT,
                    ),
                    ParamSpecString::new(
                        "page-name",
                        "Page Name",
                        "Page Name",
                        Some("welcome"),
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT,
                    ),
                ]
            });
            PROPS.as_ref()
        }
        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "label" => self.label.label().to_value(),
                "page-name" => self.name.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "label" => {
                    self.label.set_text(&value.get::<String>().unwrap());
                }
                "page-name" => {
                    self.name
                        .borrow_mut()
                        .replace(value.get::<String>().unwrap());
                }
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for SidebarRow {}
    impl ListBoxRowImpl for SidebarRow {}
}

glib::wrapper! {
    pub struct SidebarRow(ObjectSubclass<imp::SidebarRow>) @extends gtk::Widget, gtk::ListBoxRow;
}

impl SidebarRow {
    #[allow(clippy::new_without_default)]
    pub fn new(label: &str, page_name: &str) -> Self {
        glib::Object::new(&[("label", &label), ("page-name", &page_name)])
            .expect("Failed to create a SidebarRow")
    }

    pub fn title(&self) -> Option<String> {
        self.property("label")
    }

    pub fn name(&self) -> String {
        self.property::<Option<String>>("page-name")
            .unwrap_or_else(|| "welcome".to_string())
    }
}

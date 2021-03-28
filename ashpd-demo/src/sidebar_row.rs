use gtk::glib;
use gtk::prelude::*;

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, Value};
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

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
                    ParamSpec::string(
                        "label",
                        "Label",
                        "Row Label",
                        Some(""),
                        ParamFlags::READWRITE,
                    ),
                    ParamSpec::string(
                        "page-name",
                        "Page Name",
                        "Page Name",
                        Some("welcome"),
                        ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPS.as_ref()
        }
        fn get_property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.get_name() {
                "label" => self.label.get_label().to_value(),
                "page-name" => self.name.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.get_name() {
                "label" => {
                    self.label
                        .set_text(&value.get::<String>().unwrap().unwrap());
                }
                "page-name" => {
                    self.name
                        .borrow_mut()
                        .replace(value.get::<String>().unwrap().unwrap());
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
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a SidebarRow")
    }

    pub fn title(&self) -> Option<String> {
        self.get_property("label")
            .unwrap()
            .get::<String>()
            .unwrap()
    }

    pub fn name(&self) -> String {
        self.get_property("page-name")
            .unwrap()
            .get::<String>()
            .unwrap()
            .unwrap_or_else(|| "welcome".to_string())
    }
}

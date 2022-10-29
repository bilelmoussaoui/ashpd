use gtk::{glib, prelude::*};

mod imp {
    use std::cell::RefCell;

    use glib::{ParamSpec, ParamSpecString, Value};
    use gtk::subclass::prelude::*;
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/sidebar_row.ui")]
    pub struct SidebarRow {
        #[template_child]
        pub title: TemplateChild<gtk::Label>,
        pub name: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarRow {
        const NAME: &'static str = "SidebarRow";
        type Type = super::SidebarRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for SidebarRow {
        fn properties() -> &'static [ParamSpec] {
            static PROPS: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("title").construct().build(),
                    ParamSpecString::builder("page-name")
                        .default_value(Some("welcome"))
                        .construct()
                        .build(),
                ]
            });
            PROPS.as_ref()
        }
        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "title" => self.title.label().to_value(),
                "page-name" => self.name.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "title" => {
                    self.title.set_text(&value.get::<String>().unwrap());
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
    pub struct SidebarRow(ObjectSubclass<imp::SidebarRow>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl SidebarRow {
    #[allow(clippy::new_without_default)]
    pub fn new(title: &str, page_name: &str) -> Self {
        glib::Object::new(&[("title", &title), ("page-name", &page_name)])
    }

    pub fn title(&self) -> Option<String> {
        self.property("title")
    }

    pub fn name(&self) -> String {
        self.property("page-name")
    }
}

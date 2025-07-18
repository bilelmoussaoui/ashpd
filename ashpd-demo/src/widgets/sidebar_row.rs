use gtk::{glib, prelude::*};

mod imp {
    use std::{cell::RefCell, marker::PhantomData};

    use gtk::subclass::prelude::*;

    use super::*;

    #[derive(Debug, glib::Properties, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/sidebar_row.ui")]
    #[properties(wrapper_type = super::SidebarRow)]
    pub struct SidebarRow {
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        #[property(
            type = String,
            get = |r: &Self| r.title_label.label().to_string(),
            set = Self::set_title,
            construct)]
        title: PhantomData<String>,
        #[property(name = "page-name", get, set, construct, default = "welcome")]
        pub name: RefCell<String>,
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

    #[glib::derived_properties]
    impl ObjectImpl for SidebarRow {}
    impl WidgetImpl for SidebarRow {}
    impl ListBoxRowImpl for SidebarRow {}

    impl SidebarRow {
        fn set_title(&self, title: &str) {
            self.title_label.set_text(title);
        }
    }
}

glib::wrapper! {
    pub struct SidebarRow(ObjectSubclass<imp::SidebarRow>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible, gtk::Actionable;
}

impl SidebarRow {
    pub fn new(title: &str, page_name: &str) -> Self {
        glib::Object::builder()
            .property("title", title)
            .property("page-name", page_name)
            .build()
    }
}

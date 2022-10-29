use gtk::{gdk, glib, graphene, prelude::*, subclass::prelude::*};

mod imp {
    use std::cell::RefCell;

    use glib::{ParamSpec, ParamSpecBoxed};
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug, Default)]
    pub struct ColorWidget {
        pub rgba: RefCell<Option<gdk::RGBA>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ColorWidget {
        const NAME: &'static str = "ColorWidget";
        type Type = super::ColorWidget;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("color");
        }
    }

    impl ObjectImpl for ColorWidget {
        fn properties() -> &'static [ParamSpec] {
            static PROPS: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecBoxed::builder::<gdk::RGBA>("rgba").build()]);
            PROPS.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "rgba" => self.rgba.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "rgba" => {
                    self.rgba.borrow_mut().replace(value.get().unwrap());
                }
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            let widget = self.obj();
            widget.set_size_request(60, 30);
            widget.set_overflow(gtk::Overflow::Hidden);
        }
    }
    impl WidgetImpl for ColorWidget {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let color = self.rgba.borrow().unwrap_or_else(|| {
                gdk::RGBA::builder()
                    .red(53.0 / 255.0)
                    .green(132.0 / 255.0)
                    .blue(228.0 / 255.0)
                    .build()
            });
            let width = widget.width() as f32;
            let height = widget.height() as f32;
            snapshot.append_color(&color, &graphene::Rect::new(0.0, 0.0, width, height));
        }
    }
}

glib::wrapper! {
    pub struct ColorWidget(ObjectSubclass<imp::ColorWidget>)
        @extends gtk::Widget;
}

impl ColorWidget {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn set_rgba(&self, rgba: gdk::RGBA) {
        self.set_property("rgba", &rgba);
        self.queue_draw();
    }
}

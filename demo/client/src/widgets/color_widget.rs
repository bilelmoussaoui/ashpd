use gtk::{gdk, glib, graphene, prelude::*, subclass::prelude::*};

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, glib::Properties, Default)]
    #[properties(wrapper_type = super::ColorWidget)]
    pub struct ColorWidget {
        #[property(get, set = Self::set_rgba)]
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

    #[glib::derived_properties]
    impl ObjectImpl for ColorWidget {
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

    impl ColorWidget {
        pub fn set_rgba(&self, rgba: gdk::RGBA) {
            self.rgba.replace(Some(rgba));
            self.obj().queue_draw();
        }
    }
}

glib::wrapper! {
    pub struct ColorWidget(ObjectSubclass<imp::ColorWidget>)
        @extends gtk::Widget;
}

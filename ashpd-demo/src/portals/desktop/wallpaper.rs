use ashpd::zbus;
use ashpd::{RequestProxy, Response, WindowIdentifier};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use std::cell::RefCell;
    use adw::subclass::prelude::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/wallpaper.ui")]
    pub struct WallpaperPage {}

    #[glib::object_subclass]
    impl ObjectSubclass for WallpaperPage {
        const NAME: &'static str = "WallpaperPage";
        type Type = super::WallpaperPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("wallpaper.select", None, move |page, _action, _target| {
                //page.pick_color().unwrap();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for WallpaperPage {}
    impl WidgetImpl for WallpaperPage {}
    impl BinImpl for WallpaperPage {}
}

glib::wrapper! {
    pub struct WallpaperPage(ObjectSubclass<imp::WallpaperPage>) @extends gtk::Widget, adw::Bin;
}

impl WallpaperPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a WallpaperPage")
    }
}

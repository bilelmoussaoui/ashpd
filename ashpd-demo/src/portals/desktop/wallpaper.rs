use std::{cell::RefCell, str::FromStr};

use adw::prelude::*;
use ashpd::{desktop::wallpaper, Response, WindowIdentifier};
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/wallpaper.ui")]
    pub struct WallpaperPage {
        pub dialog: RefCell<Option<gtk::FileChooserNative>>,
        #[template_child]
        pub preview_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub set_on_combo: TemplateChild<adw::ComboRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WallpaperPage {
        const NAME: &'static str = "WallpaperPage";
        type Type = super::WallpaperPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("wallpaper.select", None, move |page, _action, _target| {
                page.pick_wallpaper();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for WallpaperPage {
        fn constructed(&self, _obj: &Self::Type) {
            let model = gtk::StringList::new(&["Background", "Lockscreen", "Both"]);
            self.set_on_combo.set_model(Some(&model));
        }
    }
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

    pub fn pick_wallpaper(&self) {
        let self_ = imp::WallpaperPage::from_instance(self);
        let file_chooser = gtk::FileChooserNativeBuilder::new()
            .accept_label("Select")
            .action(gtk::FileChooserAction::Open)
            .modal(true)
            .build();
        let filter = gtk::FileFilter::new();
        filter.add_pixbuf_formats();
        filter.set_name(Some("images"));
        file_chooser.add_filter(&filter);

        let show_preview = self_.preview_switch.get().get_active();
        let selected_item = self_.set_on_combo.get_selected_item().unwrap();
        let set_on = wallpaper::SetOn::from_str(
            &selected_item
                .downcast_ref::<gtk::StringObject>()
                .unwrap()
                .get_string(),
        )
        .unwrap();
        let root = self.get_root().unwrap();

        file_chooser.connect_response(clone!(@weak root => move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                let wallpaper_uri = dialog.get_file().unwrap().get_uri();
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak root => async move {
                    let identifier = WindowIdentifier::from_window(&root).await;
                    if let Ok(Response::Ok(color)) = wallpaper::set_from_uri(
                        identifier,
                        &wallpaper_uri,
                        show_preview,
                        set_on,
                    )
                    .await
                    {
                        // TODO: display a notification saying the action went fine
                        println!("{:#?}", color);
                    }
                }));
            };
            dialog.destroy();
        }));
        file_chooser.show();
        self_.dialog.replace(Some(file_chooser));
    }
}

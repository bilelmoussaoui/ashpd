use std::cell::RefCell;

use adw::prelude::*;
use ashpd::{desktop::wallpaper, WindowIdentifier};
use glib::clone;
use gtk::{glib, subclass::prelude::*};

use crate::widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl};

mod imp {
    use adw::subclass::prelude::*;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
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
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("wallpaper.select", None, move |page, _action, _target| {
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak page => async move {
                    page.pick_wallpaper().await;
                }));
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for WallpaperPage {}
    impl WidgetImpl for WallpaperPage {}
    impl BinImpl for WallpaperPage {}
    impl PortalPageImpl for WallpaperPage {}
}

glib::wrapper! {
    pub struct WallpaperPage(ObjectSubclass<imp::WallpaperPage>) @extends gtk::Widget, adw::Bin, PortalPage;
}

impl WallpaperPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a WallpaperPage")
    }

    async fn pick_wallpaper(&self) {
        let imp = self.imp();
        let root = self.native().unwrap();

        let file_chooser = gtk::FileChooserNative::builder()
            .accept_label("Select")
            .action(gtk::FileChooserAction::Open)
            .modal(true)
            .transient_for(root.downcast_ref::<gtk::Window>().unwrap())
            .build();
        let filter = gtk::FileFilter::new();
        filter.add_pixbuf_formats();
        filter.set_name(Some("images"));
        file_chooser.add_filter(&filter);

        let show_preview = imp.preview_switch.is_active();
        let set_on = match imp.set_on_combo.selected() {
            0 => wallpaper::SetOn::Background,
            1 => wallpaper::SetOn::Lockscreen,
            2 => wallpaper::SetOn::Both,
            _ => unimplemented!(),
        };
        if file_chooser.run_future().await == gtk::ResponseType::Accept {
            let identifier = WindowIdentifier::from_native(&root).await;
            match url::Url::parse(&file_chooser.file().unwrap().uri()) {
                Ok(wallpaper_uri) => {
                    match wallpaper::WallpaperRequest::default()
                        .identifier(identifier)
                        .show_preview(show_preview)
                        .set_on(set_on)
                        .build_uri(&wallpaper_uri)
                        .await
                    {
                        Err(err) => {
                            tracing::error!("Failed to set wallpaper {}", err);
                            self.send_notification(
                                "Request to set a wallpaper failed",
                                NotificationKind::Error,
                            );
                        }
                        Ok(_) => self.send_notification(
                            "Set a wallpaper request was successful",
                            NotificationKind::Success,
                        ),
                    }
                }
                Err(_err) => {
                    self.send_notification("Wallpaper URI malformed", NotificationKind::Error);
                }
            }
        };
        file_chooser.destroy();
        imp.dialog.replace(Some(file_chooser));
    }
}

use adw::{prelude::*, subclass::prelude::*};
use ashpd::{desktop::wallpaper, WindowIdentifier};
use gtk::{gio, glib};

use crate::widgets::{PortalPage, PortalPageExt, PortalPageImpl};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/wallpaper.ui")]
    pub struct WallpaperPage {
        #[template_child]
        pub preview_switch: TemplateChild<adw::SwitchRow>,
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

            klass.install_action_async("wallpaper.select", None, |page, _, _| async move {
                page.pick_wallpaper().await;
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
    pub struct WallpaperPage(ObjectSubclass<imp::WallpaperPage>)
        @extends gtk::Widget, adw::Bin, PortalPage,
        @implements gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl WallpaperPage {
    async fn open_file(&self) -> anyhow::Result<url::Url> {
        let root = self.native().unwrap();
        let filter = gtk::FileFilter::new();
        filter.add_pixbuf_formats();
        filter.set_name(Some("images"));

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);

        let dialog = gtk::FileDialog::builder()
            .modal(true)
            .accept_label("Select")
            .filters(&filters)
            .build();

        let file = dialog
            .open_future(root.downcast_ref::<gtk::Window>())
            .await?;
        let uri = url::Url::parse(&file.uri())?;
        Ok(uri)
    }

    async fn pick_wallpaper(&self) {
        let imp = self.imp();
        let root = self.native().unwrap();
        let uri = match self.open_file().await {
            Ok(uri) => uri,
            Err(err) => {
                tracing::error!("Failed to open a file: {err}");
                self.error("Failed to open a file");
                return;
            }
        };

        let show_preview = imp.preview_switch.is_active();
        let set_on = match imp.set_on_combo.selected() {
            0 => wallpaper::SetOn::Background,
            1 => wallpaper::SetOn::Lockscreen,
            2 => wallpaper::SetOn::Both,
            _ => unimplemented!(),
        };
        let identifier = WindowIdentifier::from_native(&root).await;
        match wallpaper::WallpaperRequest::default()
            .identifier(identifier)
            .show_preview(show_preview)
            .set_on(set_on)
            .build_uri(&uri)
            .await
        {
            Err(err) => {
                tracing::error!("Failed to set wallpaper: {err}");
                self.error("Request to set a wallpaper failed");
            }
            Ok(_) => self.success("Set a wallpaper request was successful"),
        }
    }
}

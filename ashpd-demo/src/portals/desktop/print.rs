use std::os::unix::prelude::AsRawFd;

use adw::subclass::prelude::*;
use ashpd::{
    desktop::print::{PageSetup, PrintProxy, Settings},
    WindowIdentifier,
};
use gtk::{gio, glib, prelude::*};

use crate::widgets::{PortalPage, PortalPageExt, PortalPageImpl};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/print.ui")]
    pub struct PrintPage {
        #[template_child]
        pub title: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub modal_switch: TemplateChild<gtk::Switch>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PrintPage {
        const NAME: &'static str = "PrintPage";
        type Type = super::PrintPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async("print.select_file", None, |page, _, _| async move {
                if let Err(err) = page.select_file().await {
                    tracing::error!("Failed to pick a file {err}");
                }
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for PrintPage {}
    impl WidgetImpl for PrintPage {}
    impl BinImpl for PrintPage {}
    impl PortalPageImpl for PrintPage {}
}

glib::wrapper! {
    pub struct PrintPage(ObjectSubclass<imp::PrintPage>)
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl PrintPage {
    async fn select_file(&self) -> anyhow::Result<()> {
        let imp = self.imp();
        let title = imp.title.text();
        let modal = imp.modal_switch.is_active();
        let root = self.native().unwrap();

        let filter = gtk::FileFilter::new();
        filter.add_pixbuf_formats();
        filter.set_name(Some("images"));

        let filters = gio::ListStore::new(gtk::FileFilter::static_type());
        filters.append(&filter);

        let dialog = gtk::FileDialog::builder()
            .accept_label("Select")
            .modal(true)
            .filters(&filters)
            .build();

        let path = dialog
            .open_future(root.downcast_ref::<gtk::Window>())
            .await?
            .path()
            .unwrap();
        let file = std::fs::File::open(path).unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;

        match print(&identifier, &title, file, modal).await {
            Ok(_) => {
                self.success("Print request was successful");
            }
            Err(err) => {
                tracing::error!("Failed to print {}", err);
                self.error("Request to print failed");
            }
        }
        Ok(())
    }
}

async fn print<F: AsRawFd>(
    identifier: &WindowIdentifier,
    title: &str,
    file: F,
    modal: bool,
) -> ashpd::Result<()> {
    let proxy = PrintProxy::new().await?;

    let out = proxy
        .prepare_print(
            identifier,
            title,
            Settings::default(),
            PageSetup::default(),
            modal,
        )
        .await?
        .response()?;

    proxy
        .print(identifier, title, &file, Some(out.token), modal)
        .await?;

    Ok(())
}
